/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Tauri Commands — transport layer only.
//!
//! Rule: no command touches llm_engine or tts_engine directly.
//! All logic lives in SessionManager.

use std::sync::Arc;

use crate::{
    audio::capture::AudioCapture, audio::list_input_devices, config::settings_loader::SettingsLoader,
    errors::AppError, modes::ModeFileInfo, schemas::{AiState, PersistentSettings}, state::AppState,
};
use serde::{Deserialize, Serialize};
use tauri::State;

// ─── Response Structs ────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContextUsage {
    pub used: u32,
    pub total: u32,
    pub percent: f32,
}

// ─── Session ─────────────────────────────────────────────────

/// Loads a mode, starts a session, and if llm_initiates=true, triggers the
/// opening line before responding to the frontend.
///
/// Key pattern:
///   1. Lock SessionManager → prepare session + obtain prompt → DROP the lock
///   2. Clone Arc<Mutex<SessionManager>> → call run_turn_streaming
///      (SessionManager already has the engines; no State<'_> in the spawn)
#[tauri::command]
pub async fn start_session(
    path: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<crate::session::SessionInfo, String> {
    let (session_info, turn_prompt) = {
        let mut manager = state.session_manager.lock().map_err(|e| e.to_string())?;
        let info = manager.start_session(&path).map_err(|e| e.to_string())?;
        let should_infer = info.caps.llm_initiates;
        let prompt = if should_infer {
            Some(manager.get_turn_prompt("", false).unwrap_or_default())
        } else {
            None
        };
        (info, prompt)
    };

    if let Some(prompt) = turn_prompt {
        let manager = state.session_manager.lock().map_err(|e| e.to_string())?;
        manager.set_ai_state(AiState::Thinking, Some(&app));
        manager.run_turn_streaming(None, prompt, app);
    }

    Ok(session_info)
}

#[tauri::command]
pub fn end_session(state: State<'_, AppState>) -> Result<crate::session::SessionSummary, String> {
    let mut manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    manager.end_session().map_err(|e| e.to_string())
}

// ─── Audio ───────────────────────────────────────────────────

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    list_input_devices()
        .map(|devices| {
            devices
                .into_iter()
                .map(|d| AudioDevice {
                    name: d.name,
                    id: String::new(),
                })
                .collect()
        })
        .map_err(|e: AppError| e.to_string())
}

#[tauri::command]
pub fn start_listening(
    device_name: Option<String>,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut capture = match device_name.as_deref() {
        Some(name) => AudioCapture::with_device_name(name),
        None => AudioCapture::default(),
    }
    .map_err(|e| e.to_string())?;

    let sample_rate = capture.sample_rate();
    capture.start().map_err(|e| e.to_string())?;

    let mut capture_lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut prev) = *capture_lock {
        let _ = prev.stop();
    }
    *capture_lock = Some(capture);
    drop(capture_lock);

    if let Ok(mut hz) = state.sample_rate.lock() {
        *hz = sample_rate;
    }

    let manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    manager.set_ai_state(AiState::Listening, Some(&app));

    Ok(())
}

/// Stops capture and launches streaming inference.
/// SessionManager provides the engines; no direct access here.
#[tauri::command]
pub async fn stop_and_process_streaming(
    prompt: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // ── Phase 1: extract audio (synchronous) ─────────────────────
    let (audio_raw, src_hz) = {
        let mut lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
        let capture = lock.as_mut().ok_or("No active audio capture")?;
        capture.stop().map_err(|e| e.to_string())?;
        (capture.take_buffer(), capture.sample_rate())
    };

    if audio_raw.is_empty() {
        return Err("Audio buffer empty".into());
    }

    let resampled = if src_hz != 16000 {
        crate::audio::MelPreprocessor::resample(&audio_raw, src_hz as f32, 16000.0)
    } else {
        audio_raw
    };

    // ── Phase 2: delegate to SessionManager ─────────────────────
    // Arc cloned before any await — State<'_> free
    let manager_arc = Arc::clone(&state.session_manager);
    {
        let mut manager = manager_arc.lock().map_err(|e| e.to_string())?;
        let fullprompt = manager.get_turn_prompt(&prompt, true).unwrap_or_default();
        manager.set_ai_state(AiState::Thinking, Some(&app));
        manager.run_turn_streaming(Some(resampled), fullprompt, app);
    }

    Ok(())
}

#[tauri::command]
pub async fn process_text_streaming(
    prompt: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Arc cloned before any await — State<'_> free
    let manager_arc = Arc::clone(&state.session_manager);
    {
        let mut manager = manager_arc.lock().map_err(|e| e.to_string())?;
        let fullprompt = manager.get_turn_prompt(&prompt, false).unwrap_or_default();
        manager.set_ai_state(AiState::Thinking, Some(&app));
        manager.run_turn_streaming(None, fullprompt, app);
    }

    Ok(())
}

// ─── TTS ─────────────────────────────────────────────────────

#[tauri::command]
pub fn list_voices(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    let tts = manager.tts_engine.lock().map_err(|e| e.to_string())?;
    Ok(tts.as_ref().map(|e| e.list_voices()).unwrap_or_default())
}

// ─── LLM / Debug ─────────────────────────────────────────────

#[tauri::command]
pub fn reset_conversation(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    let mut llm = manager.llm_engine.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut model) = *llm {
        model
            .reset()
            .map(|_| {
                eprintln!("🧹 Conversation reset from frontend");
                true
            })
            .map_err(|e| e.to_string())
    } else {
        Ok(true)
    }
}

#[tauri::command]
pub fn get_context_usage(state: State<'_, AppState>) -> Result<ContextUsage, String> {
    let manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    let engine = manager.llm_engine.lock().map_err(|e| e.to_string())?;

    if let Some(ref engine) = *engine {
        let (used, total) = engine.model().context_usage(engine.state().n_past);
        Ok(ContextUsage {
            used,
            total,
            percent: if total > 0 {
                used as f32 / total as f32 * 100.0
            } else {
                0.0
            },
        })
    } else {
        Ok(ContextUsage {
            used: 0,
            total: 4096,
            percent: 0.0,
        })
    }
}

// ─── Settings ──────────────────────────────────────────────────

#[tauri::command]
pub fn load_settings() -> Result<PersistentSettings, String> {
    let loader = SettingsLoader::new();
    loader.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(settings: PersistentSettings) -> Result<(), String> {
    let loader = SettingsLoader::new();
    loader.save(&settings).map_err(|e| e.to_string())
}

// ─── Misc ────────────────────────────────────────────────────

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn get_list_modes() -> Result<Vec<ModeFileInfo>, String> {
    crate::modes::list_modes().map_err(|e| e.to_string())
}

// ─── Private helpers ─────────────────────────────────────────
