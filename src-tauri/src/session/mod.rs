/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Lifecycle of a practice session.
//!
//! SessionManager is the single owner of session state, LLM, and TTS engines.
//! Commands are thin transport layer — they clone the Arc<Mutex<SessionManager>>
//! and call methods here, never touching engines directly.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use uuid::Uuid;

use crate::{
    errors::{AppResult, SessionError},
    llama::InferenceEngine, // adjust to your actual types
    modes::{load_mode, ModeConfig},
    schemas::{master_system_instruction, BabiloAnalysis, BabiloEvent, TokenEvent},
    tts::TtsEngine, // adjust to your actual types
};

// ─── Session States ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Initialized,
    Active,
    Paused,
    Ended,
}

// ─── Structs sent to frontend ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCaps {
    pub accepts_audio: bool,
    pub accepts_text: bool,
    pub llm_initiates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub mode_id: String,
    pub mode_name: String,
    pub caps: SessionCaps,
    /// First LLM line when llm_initiates=true; None if user speaks first.
    pub opening_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub mode_name: String,
    pub turns: u32,
    pub average_score: u8,
}

// ─── Prompt helpers ──────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PromptHierarchy {
    pub system_format: &'static str,
    pub mode_prompt: String,
    pub role_prompt: String,
}

#[derive(Clone, Debug, Default)]
pub struct PromptInjectionState {
    pub turns_processed: u32,
    pub role_injected: bool,
}

impl PromptInjectionState {
    pub fn should_remind_system(&self, interval: u32) -> bool {
        self.turns_processed > 0 && self.turns_processed % interval == 0
    }
    pub fn increment(&mut self) {
        self.turns_processed += 1;
    }
}

const SYSTEM_REMINDER_INTERVAL: u32 = 5;

// ─── Internal Session ────────────────────────────────────────

pub struct Session {
    pub id: String,
    pub mode: Arc<dyn ModeConfig>,
    pub state: SessionState,
    pub turns: u32,
    pub scores: Vec<u8>,
    pub hierarchy: PromptHierarchy,
    pub injection: PromptInjectionState,
}

// ─── SessionManager ──────────────────────────────────────────
//
// Owns the engine Arcs so commands never need to touch them directly.
// This is the key change: LLM and TTS live here, not loose in AppState.

pub struct SessionManager {
    active_session: Option<Session>,
    // Engines are kept as Arc so spawn_blocking can clone them cheaply
    // without holding a borrow on SessionManager itself.
    pub llm_engine: Arc<Mutex<Option<InferenceEngine>>>,
    pub tts_engine: Arc<Mutex<Option<TtsEngine>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: None,
            llm_engine: Arc::new(Mutex::new(None)),
            tts_engine: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load_engines(&mut self, llm: Option<InferenceEngine>, tts: Option<TtsEngine>) {
        *self.llm_engine.lock().unwrap() = llm;
        *self.tts_engine.lock().unwrap() = tts;
    }

    // ── Session lifecycle ────────────────────────────────────

    pub fn start_session(&mut self, path: &str) -> AppResult<SessionInfo> {
        if self.active_session.is_some() {
            return Err(SessionError::AlreadyActive.into());
        }

        // Reset LLM context for a clean slate
        {
            let mut llm = self
                .llm_engine
                .lock()
                .map_err(|e| SessionError::LockError(e.to_string()))?;
            if let Some(ref mut engine) = *llm {
                engine
                    .reset()
                    .map_err(|e| SessionError::LoadError(e.to_string()))?;
            }
        }

        let mode = Arc::new(load_mode(path).map_err(|e| SessionError::LoadError(e.to_string()))?);
        let session_id = Uuid::new_v4().to_string();
        let hierarchy =
            get_prompt_hierarchy(mode.as_ref()).map_err(SessionError::PromptComposition)?;

        let session = Session {
            id: session_id.clone(),
            mode,
            state: SessionState::Initialized,
            turns: 0,
            scores: Vec::new(),
            hierarchy,
            injection: PromptInjectionState::default(),
        };

        let session_info = build_session_info(session_id, &session.mode, None);
        self.active_session = Some(session);
        Ok(session_info)
    }

    pub fn end_session(&mut self) -> AppResult<SessionSummary> {
        let session = self
            .active_session
            .take()
            .ok_or_else(|| SessionError::NotFound("no active session".into()))?;

        if session.state == SessionState::Ended {
            return Err(SessionError::InvalidStateTransition {
                from: "Ended".into(),
                to: "Ended".into(),
            }
            .into());
        }

        Ok(build_session_summary(
            session.id,
            session.mode.name().to_string(),
            session.turns,
            &session.scores,
        ))
    }

    pub fn pause_session(&mut self) -> AppResult<()> {
        let session = self.get_active_session_mut()?;
        if session.state != SessionState::Active {
            return Err(SessionError::InvalidStateTransition {
                from: format!("{:?}", session.state),
                to: "Paused".into(),
            }
            .into());
        }
        session.state = SessionState::Paused;
        Ok(())
    }

    pub fn resume_session(&mut self) -> AppResult<()> {
        let session = self.get_active_session_mut()?;
        if session.state != SessionState::Paused {
            return Err(SessionError::InvalidStateTransition {
                from: format!("{:?}", session.state),
                to: "Active".into(),
            }
            .into());
        }
        session.state = SessionState::Active;
        Ok(())
    }

    pub fn record_turn(&mut self, score: u8) -> AppResult<()> {
        let session = self.get_active_session_mut()?;
        if session.state != SessionState::Active {
            return Err(SessionError::OperationNotAllowed(
                "cannot record turn in non-active session".into(),
            )
            .into());
        }
        session.turns += 1;
        session.scores.push(score);
        Ok(())
    }

    // ── Prompt building ──────────────────────────────────────

    pub fn get_turn_prompt(&mut self, user_input: &str, is_audio: bool) -> AppResult<String> {
        let session = self
            .active_session
            .as_mut()
            .ok_or(SessionError::NotInitialized)?;
        Ok(build_turn_prompt(
            &session.hierarchy,
            &mut session.injection,
            user_input,
            is_audio,
        ))
    }

    // ── Inference (owns the engines, no State<'_> needed) ────
    //
    // Call this from commands after releasing any lock on SessionManager.
    // Pattern:
    //   1. Lock SessionManager → get turn_prompt → drop lock
    //   2. Clone Arc<Mutex<SessionManager>> and call run_turn_streaming

    pub fn run_turn_streaming(&self, audio_raw: Vec<f32>, prompt: String, app: tauri::AppHandle) {
        // Clone Arcs — no borrow of self crosses the spawn boundary
        let llm_engine = Arc::clone(&self.llm_engine);
        let tts_engine = Arc::clone(&self.tts_engine);

        tauri::async_runtime::spawn_blocking(move || {
            let emit = |event: BabiloEvent| {
                let _ = app.emit("babilo://stream", &event);
            };

            let (tts_tx, tts_rx) = std::sync::mpsc::channel::<String>();

            let tts_handle = std::thread::spawn(move || {
                let mut lock = tts_engine.lock().unwrap();
                let tts = match lock.as_mut() {
                    Some(t) => t,
                    None => return,
                };
                while let Ok(sentence) = tts_rx.recv() {
                    let _ = tts.speak_and_play(&sentence, "F1", "en", 1.5, 30);
                }
            });

            let mut llm_lock = llm_engine.lock().unwrap();
            match llm_lock.as_mut() {
                None => emit(BabiloEvent::Error {
                    message: "LLM not initialized".into(),
                }),
                Some(model) => {
                    if model.model().context_is_full(model.state().n_past, 256) {
                        let _ = model.reset();
                    }

                    let mut analysis_buf = String::new();
                    let mut response_buf = String::new();
                    let mut tts_tx_opt = Some(tts_tx);

                    let result =
                        model.infer_audio_streaming(&audio_raw, &prompt, |event| match event {
                            TokenEvent::ResponseToken(text) => {
                                response_buf.push_str(&text);
                            }
                            TokenEvent::SentinelReached => {
                                let partial = response_buf.trim().to_string();
                                if !partial.is_empty() {
                                    if let Some(tx) = &tts_tx_opt {
                                        let _ = tx.send(partial);
                                    }
                                }
                                emit(BabiloEvent::SentinelReached);
                            }
                            TokenEvent::AnalysisToken(text) => {
                                analysis_buf.push_str(&text);
                            }
                            TokenEvent::Done => {
                                drop(tts_tx_opt.take());
                                let final_response = response_buf.trim().to_string();
                                if final_response.is_empty() {
                                    emit(BabiloEvent::Error {
                                        message: "Empty response from model".into(),
                                    });
                                    return;
                                }
                                match BabiloAnalysis::builder()
                                    .with_response(final_response)
                                    .with_json_payload(&analysis_buf)
                                    .and_then(|b| b.build())
                                {
                                    Ok(data) => emit(BabiloEvent::Analysis { data }),
                                    Err(e) => emit(BabiloEvent::Error {
                                        message: e.to_string(),
                                    }),
                                }
                            }
                        });

                    if let Err(e) = result {
                        emit(BabiloEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }

            let _ = tts_handle.join();
        });
    }

    // ── Internal helpers ─────────────────────────────────────

    pub fn get_active_session(&self) -> AppResult<&Session> {
        self.active_session
            .as_ref()
            .ok_or_else(|| SessionError::NotInitialized.into())
    }

    pub fn get_active_session_mut(&mut self) -> AppResult<&mut Session> {
        self.active_session
            .as_mut()
            .ok_or_else(|| SessionError::NotInitialized.into())
    }

    pub fn require_mode(&self) -> AppResult<Arc<dyn ModeConfig>> {
        self.active_session
            .as_ref()
            .map(|s| Arc::clone(&s.mode))
            .ok_or_else(|| SessionError::NotInitialized.into())
    }
}

// ─── Free helpers ────────────────────────────────────────────

pub fn get_prompt_hierarchy(mode: &dyn ModeConfig) -> Result<PromptHierarchy, String> {
    Ok(PromptHierarchy {
        system_format: master_system_instruction(),
        mode_prompt: mode.mode_prompt().to_string(),
        role_prompt: mode.role_prompt().unwrap_or("").to_string(),
    })
}

pub fn build_session_info(
    session_id: String,
    mode: &Arc<dyn ModeConfig>,
    opening_line: Option<String>,
) -> SessionInfo {
    SessionInfo {
        session_id,
        mode_id: mode.id().to_string(),
        mode_name: mode.name().to_string(),
        caps: SessionCaps {
            accepts_audio: mode.accepts_audio(),
            accepts_text: mode.accepts_text(),
            llm_initiates: mode.llm_initiates(),
        },
        opening_line,
    }
}

pub fn build_session_summary(
    session_id: String,
    mode_name: String,
    turns: u32,
    scores: &[u8],
) -> SessionSummary {
    let average_score = if scores.is_empty() {
        0
    } else {
        let sum: u32 = scores.iter().map(|&s| s as u32).sum();
        (sum / scores.len() as u32) as u8
    };
    SessionSummary {
        session_id,
        mode_name,
        turns,
        average_score,
    }
}

pub fn build_turn_prompt(
    hierarchy: &PromptHierarchy,
    injection: &mut PromptInjectionState,
    user_input: &str,
    is_audio: bool,
) -> String {
    let mut parts = Vec::with_capacity(5);

    if !injection.role_injected {
        parts.push(format!("[ROLE]\n{}", hierarchy.role_prompt));
        injection.role_injected = true;
    }
    if injection.turns_processed == 0 {
        parts.push(format!("[MODE]\n{}", hierarchy.mode_prompt));
    }
    if injection.turns_processed == 0 || injection.should_remind_system(SYSTEM_REMINDER_INTERVAL) {
        parts.push(format!("[SYSTEM FORMAT]\n{}", hierarchy.system_format));
    }

    parts.push(format!("[USER]\n{}", user_input.trim()));

    let content = parts.join("\n\n");
    let base = format!("<|turn|>user\n{content}<|turn|>\n<|turn|>model\n");

    if is_audio {
        let marker = llama_cpp_2::mtmd::mtmd_default_marker();
        let json_reminder =
            "\n[REMINDER: After conversational reply, output <|babilo_analysis|> followed by valid JSON.]";
        if let Some(pos) = base.find("<|turn|>user\n") {
            let insert = pos + "<|turn|>user\n".len();
            let (before, after) = base.split_at(insert);
            return format!("{before}{marker}{after}{json_reminder}");
        }
    }

    injection.increment();

    base
}
