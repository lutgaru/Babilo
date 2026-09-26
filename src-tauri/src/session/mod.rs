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
    llama::InferenceEngine,
    modes::{load_mode, ModeConfig},
    schemas::{
        analysis_system_instruction, conversation_system_instruction, AiState, BabiloAnalysis,
        BabiloEvent, SessionReport, TokenEvent,
    },
    tts::TtsEngine,
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
    /// Whether the system prompt was injected when building the last turn prompt.
    pub last_sys_injected: bool,
    /// Total number of times the system prompt has been injected in this session.
    pub sys_injection_total: u32,
}

/// Snapshot of per-turn prompt metadata passed into `run_turn_streaming`.
/// Captured under the SessionManager lock in commands, before the lock is
/// dropped and inference runs on a background thread.
#[derive(Debug, Clone, Copy)]
pub struct TurnReportMeta {
    pub turn: u32,
    pub sys_prompt_injected: bool,
    pub sys_prompt_injections: u32,
}

// ─── SessionManager ──────────────────────────────────────────

pub struct SessionManager {
    active_session: Option<Session>,
    pub llm_engine: Arc<Mutex<Option<InferenceEngine>>>,
    pub tts_engine: Arc<Mutex<Option<TtsEngine>>>,
    ai_state: Arc<Mutex<AiState>>,
    /// When true, audio embeddings in the main context are replaced by
    /// their transcription after each turn (optimize conversation).
    /// When false, full audio tokens are kept (maintain all information).
    optimize_audio_cache: bool,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: None,
            llm_engine: Arc::new(Mutex::new(None)),
            tts_engine: Arc::new(Mutex::new(None)),
            ai_state: Arc::new(Mutex::new(AiState::Idle)),
            optimize_audio_cache: true,
        }
    }

    /// Live-update the audio cache strategy (from settings, no restart needed).
    pub fn set_optimize_audio_cache(&mut self, value: bool) {
        self.optimize_audio_cache = value;
    }

    pub fn optimize_audio_cache(&self) -> bool {
        self.optimize_audio_cache
    }

    pub fn load_engines(&mut self, llm: Option<InferenceEngine>, tts: Option<TtsEngine>) {
        *self.llm_engine.lock().unwrap() = llm;
        *self.tts_engine.lock().unwrap() = tts;
    }

    pub fn set_ai_state(&self, state: AiState, app: Option<&tauri::AppHandle>) {
        *self.ai_state.lock().unwrap() = state;
        if let Some(app) = app {
            let _ = app.emit("babilo://ai-state", &state);
        }
    }

    pub fn get_ai_state(&self) -> AiState {
        *self.ai_state.lock().unwrap()
    }

    pub fn ai_state_arc(&self) -> Arc<Mutex<AiState>> {
        Arc::clone(&self.ai_state)
    }

    // ── Session lifecycle ────────────────────────────────────

    pub fn start_session(&mut self, path: &str) -> AppResult<SessionInfo> {
        if self.active_session.is_some() {
            return Err(SessionError::AlreadyActive.into());
        }

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
            last_sys_injected: false,
            sys_injection_total: 0,
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
        let (prompt, injected) = build_turn_prompt(
            &session.hierarchy,
            &mut session.injection,
            user_input,
            is_audio,
        );
        session.last_sys_injected = injected;
        if injected {
            session.sys_injection_total += 1;
        }
        Ok(prompt)
    }

    /// Capture the per-turn report metadata for the last built prompt.
    /// Call right after `get_turn_prompt`, while holding the manager lock.
    pub fn last_turn_meta(&self) -> AppResult<TurnReportMeta> {
        let session = self
            .active_session
            .as_ref()
            .ok_or(SessionError::NotInitialized)?;
        Ok(TurnReportMeta {
            turn: session.injection.turns_processed,
            sys_prompt_injected: session.last_sys_injected,
            sys_prompt_injections: session.sys_injection_total,
        })
    }

    fn build_session_report(
        meta: TurnReportMeta,
        context_used: u32,
        context_total: u32,
        context_percent: f32,
        context_cleaned: bool,
    ) -> SessionReport {
        SessionReport {
            turn: meta.turn,
            context_used,
            context_total,
            context_percent,
            sys_prompt_injected: meta.sys_prompt_injected,
            sys_prompt_injections: meta.sys_prompt_injections,
            context_cleaned,
        }
    }

    // ── Two-phase inference ──────────────────────────────────
    //
    // Phase 1 — Response:   uses main context (200k), generates conversational reply
    // Phase 2 — Analysis:   uses analysis context (2k, reset each turn), generates JSON
    //
    // Call from commands:
    //   1. Lock SessionManager → get_turn_prompt → drop lock
    //   2. Clone Arc and call run_turn_streaming

    pub fn run_turn_streaming(
        &self,
        audio_raw: Option<Vec<f32>>,
        prompt: String,
        user_input: String,
        report_meta: TurnReportMeta,
        app: tauri::AppHandle,
    ) {
        let llm_engine = Arc::clone(&self.llm_engine);
        let tts_engine = Arc::clone(&self.tts_engine);
        let ai_state = Arc::clone(&self.ai_state);
        // Snapshot the live setting so the background turn honors the
        // current toggle without holding the manager lock during inference.
        let optimize_audio_cache = self.optimize_audio_cache;

        let app_for_state = app.clone();
        let ai_state_changed = move |state: AiState| {
            *ai_state.lock().unwrap() = state;
            let _ = app_for_state.emit("babilo://ai-state", &state);
        };

        ai_state_changed(AiState::Thinking);

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
            let model = match llm_lock.as_mut() {
                None => {
                    emit(BabiloEvent::Error {
                        message: "LLM not initialized".into(),
                    });
                    return;
                }
                Some(m) => m,
            };

            // ── Phase 1: Generate response ──────────────────────────
            let response_text = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let tts_tx_opt = std::sync::Arc::new(std::sync::Mutex::new(Some(tts_tx)));

            let cb_response_text = response_text.clone();
            let cb_tts_tx = tts_tx_opt.clone();

            let response_callback = move |event: TokenEvent| match event {
                TokenEvent::Token(text) => {
                    cb_response_text.lock().unwrap().push_str(&text);
                }
                TokenEvent::Done => {
                    let text = cb_response_text.lock().unwrap().trim().to_string();
                    if !text.is_empty() {
                        if let Some(tx) = &*cb_tts_tx.lock().unwrap() {
                            let _ = tx.send(text);
                        }
                    }
                }
            };

            let phase1_result = match audio_raw {
                Some(ref pcm) => model.infer_audio_streaming(pcm, &prompt, response_callback),
                None => model.infer_text_streaming(&prompt, response_callback),
            };

            if let Err(e) = phase1_result {
                emit(BabiloEvent::Error {
                    message: format!("Response generation failed: {}", e),
                });
                // Still send a session report so the frontend stays in sync,
                // even when generation fails after main operations started.
                let (used, total, percent) = model.context_usage_snapshot();
                emit(BabiloEvent::SessionReport {
                    report: Self::build_session_report(
                        report_meta,
                        used,
                        total,
                        percent,
                        model.was_context_cleaned(),
                    ),
                });
                ai_state_changed(AiState::Idle);
                return;
            }

            // Emit response to frontend (TTS is already playing in background)
            let final_response = response_text.lock().unwrap().trim().to_string();
            emit(BabiloEvent::Response {
                text: final_response.clone(),
            });
            ai_state_changed(AiState::Speaking);

            // ── Phase 2: Generate analysis (while TTS plays) ──────────
            let is_audio = audio_raw.is_some();
            let analysis_prompt = build_analysis_prompt(&user_input, is_audio);
            let mut analysis_buf = String::new();

            let analysis_callback = |event: TokenEvent| match event {
                TokenEvent::Token(text) => {
                    analysis_buf.push_str(&text);
                }
                TokenEvent::Done => {}
            };

            let phase2_result = match audio_raw {
                Some(ref pcm) => {
                    model.infer_audio_analysis_streaming(pcm, &analysis_prompt, analysis_callback)
                }
                None => model.infer_analysis_streaming(&analysis_prompt, analysis_callback),
            };

            // Close TTS channel and wait for playback to finish
            drop(tts_tx_opt.lock().unwrap().take());
            let _ = tts_handle.join();

            match phase2_result {
                Ok(()) => {
                    let result = BabiloAnalysis::builder()
                        .with_json_payload(&analysis_buf)
                        .and_then(|b| b.build())
                        .map_err(|e| e.to_string());

                    match result {
                        Ok(data) => {
                            // ── Audio cache optimization (optional) ──
                            // Replace audio embeddings in the main context
                            // with the transcription so the KV cache does
                            // not fill up with audio tokens. Runs on the
                            // main context only; analysis context is reset
                            // every turn anyway.
                            // Disabled → keep full audio tokens (max fidelity).
                            if is_audio && optimize_audio_cache {
                                match model.compact_audio_turn(
                                    &prompt,
                                    &data.transcription,
                                    &final_response,
                                ) {
                                    Ok(stats) => {
                                        if stats.compacted {
                                            eprintln!(
                                                "♻️ audio compacted: {} → {} (freed {})",
                                                stats.n_past_before,
                                                stats.n_past_after,
                                                stats.tokens_freed
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ audio compaction failed: {e}");
                                    }
                                }
                            }
                            emit(BabiloEvent::Analysis { data });
                        }
                        Err(msg) => {
                            emit(BabiloEvent::Error {
                                message: format!("Analysis parse failed: {}", msg),
                            });
                        }
                    }
                }
                Err(e) => {
                    emit(BabiloEvent::Error {
                        message: format!("Analysis generation failed: {}", e),
                    });
                }
            }

            // ── Session report: always sent after main operations ──
            // Covers context usage, sys-prompt injection state, and whether
            // the KV cache was wiped to make room this turn.
            let (used, total, percent) = model.context_usage_snapshot();
            emit(BabiloEvent::SessionReport {
                report: Self::build_session_report(
                    report_meta,
                    used,
                    total,
                    percent,
                    model.was_context_cleaned(),
                ),
            });

            ai_state_changed(AiState::Idle);
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
        system_format: conversation_system_instruction(),
        mode_prompt: mode.mode_prompt().to_string(),
        role_prompt: mode.role_prompt().unwrap_or("").to_string(),
    })
}

/// Build a concise analysis prompt from the current turn data.
/// This prompt is fed to the small analysis context (reset each turn).
/// When `is_audio` is true, the MTMD marker is appended so the model
/// receives audio embeddings during analysis.
pub fn build_analysis_prompt(user_input: &str, is_audio: bool) -> String {
    let input = if is_audio {
        format!(
            "{}\n{}",
            user_input,
            llama_cpp_2::mtmd::mtmd_default_marker()
        )
    } else {
        user_input.to_string()
    };

    format!(
        r#"
<|turn>system\n
{}
\n<turn|>
<|turn>user\n
{}
\n<turn|>
<|turn>model\n
"#,
        analysis_system_instruction().trim(),
        input,
    )
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
) -> (String, bool) {
    let marker = llama_cpp_2::mtmd::mtmd_default_marker().to_string();
    let mut prompt = String::new();

    if injection.turns_processed == 0 {
        prompt.push_str("<bos>");
    }

    let injected =
        injection.turns_processed == 0 || injection.should_remind_system(SYSTEM_REMINDER_INTERVAL);
    if injected {
        let system = format!(
            "{}.{}.\n{}",
            hierarchy.mode_prompt.trim(),
            hierarchy.role_prompt.trim(),
            hierarchy.system_format.trim(),
        );
        prompt.push_str(&format!("<|turn>system\n{system}\n<turn|>\n"));
    }

    if !user_input.trim().is_empty() || is_audio {
        prompt.push_str(&format!("<|turn>user\n{user_input}\n{marker}\n<turn|>\n"));
    }

    prompt.push_str("<|turn>model\n");

    injection.increment();
    (prompt, injected)
}
