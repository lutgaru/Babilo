/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Comandos Tauri exponibles al frontend

use std::sync::Arc;

use crate::{
    audio::capture::AudioCapture,
    audio::list_input_devices,
    errors::AppError,
    modes::{ModeFileInfo},
    schemas::{BabiloAnalysis, BabiloEvent, TokenEvent},
    state::AppState,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

// ─────────────────────────────────────────────────────────────
// Structs de respuesta
// ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool,
    pub message: String,
    pub audio_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ContextUsage {
    pub used: u32,
    pub total: u32,
    pub percent: f32,
}

// ─────────────────────────────────────────────────────────────
// Comandos
// ─────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────
// Session
// ─────────────────────────────────────────────────────────────
/// Loads a mode from a .babilo.json file and starts a new session.
///
/// If llm_initiates=true, generates the opening line before responding
/// to the frontend — the user sees the mode ready to speak from the first frame.
///
/// Returns SessionInfo with caps and opening_line so the frontend
/// can configure the widgets without knowing the mode type.
#[tauri::command]
pub async fn start_session(
    path: String,
    state: State<'_, AppState>,
) -> Result<crate::session::SessionInfo, String> {
    // 1. Load the mode from JSON
    let mut manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    match manager.start_session(&path) {
        Ok(session_info) => Ok(session_info),
        Err(e) => Err(e.to_string()),
    }
}
/// Ends the active session and returns the summary.
///
/// For now, turns and scores come from the basic AppState.
/// In phase 2, AppState will accumulate scores per turn during the session.
#[tauri::command]
pub fn end_session(state: State<'_, AppState>) -> Result<crate::session::SessionSummary, String> {
    // Retrieve mode name before cleanup
    let mut manager = state.session_manager.lock().map_err(|e| e.to_string())?;
    match manager.end_session() {
        Ok(summary) => Ok(summary),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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
pub fn synthesize(
    text: String,
    voice: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<i8>, String> {
    let voice_id = voice.unwrap_or_else(|| "F1".to_string());
    let mut engine_lock = state.tts_engine.lock().map_err(|e| e.to_string())?;

    match engine_lock.as_mut() {
        None => Err("TTS engine not initialized".into()),
        Some(engine) => engine
            .speak(&text, &voice_id, "en", 1.0, 30)
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn list_voices(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let engine_lock = state.tts_engine.lock().map_err(|e| e.to_string())?;

    Ok(engine_lock
        .as_ref()
        .map(|e| e.list_voices())
        .unwrap_or_default())
}

#[tauri::command]
pub fn start_listening(
    device_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut capture = match device_name.as_deref() {
        Some(name) => AudioCapture::with_device_name(name),
        None => AudioCapture::default(),
    }
    .map_err(|e| e.to_string())?;

    // Guardamos sample_rate antes de mover capture
    let sample_rate = capture.sample_rate();
    capture.start().map_err(|e| e.to_string())?;

    // Un solo lock cubre el stop anterior + la asignación nueva
    let mut capture_lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut prev) = *capture_lock {
        let _ = prev.stop();
    }
    *capture_lock = Some(capture);
    drop(capture_lock);

    if let Ok(mut hz) = state.sample_rate.lock() {
        *hz = sample_rate;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_and_process(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
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

    let mut llm_lock = state.llm_engine.lock().map_err(|e| e.to_string())?;
    match llm_lock.as_mut() {
        Some(model) => {
            if model.model().context_is_full(model.state().n_past, 256) {
                let _ = model.reset();
            }
            model
                .infer_audio(&resampled, &prompt)
                .map_err(|e| e.to_string())
        }
        None => Ok(format!("[Echo] {}", prompt)),
    }
}

#[tauri::command]
pub async fn stop_and_process_streaming(
    prompt: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    eprint!(
        "🛑 Stopping audio capture and processing with prompt: {}",
        prompt
    );
    let (audio_raw, src_hz) = {
        let mut lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
        let capture = lock.as_mut().ok_or("No active audio capture")?;
        capture.stop().map_err(|e| e.to_string())?;
        (capture.take_buffer(), capture.sample_rate())
    };

    if audio_raw.is_empty() {
        eprintln!("⚠️ Audio buffer empty after capture");
        return Err("Audio buffer empty".into());
    }

    let resampled = if src_hz != 16000 {
        crate::audio::MelPreprocessor::resample(&audio_raw, src_hz as f32, 16000.0)
    } else {
        audio_raw
    };

    // ← Extract Arcs BEFORE spawn_blocking, while State<'_> is still in scope
    let tts_engine = Arc::clone(&state.tts_engine);
    let llm_engine = Arc::clone(&state.llm_engine);
    let app_clone = app.clone();
    eprint!("🎬 Starting inference with prompt: ");
    tauri::async_runtime::spawn_blocking(move || {
        let emit = |event: BabiloEvent| {
            let _ = app_clone.emit("babilo://stream", &event);
        };

        let (tts_tx, tts_rx) = std::sync::mpsc::channel::<String>();

        // tts_engine is an owned Arc now — no borrow of State<'_>
        let tts_handle = std::thread::spawn(move || {
            let mut lock = tts_engine.lock().unwrap();
            let tts = match lock.as_mut() {
                Some(t) => t,
                None => return,
            };
            while let Ok(sentence) = tts_rx.recv() {
                let _ = tts.speak_and_play(&sentence, "F1", "en", 1.5, 30);
                eprint!("🎤 TTS spoke: {}", sentence);
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

                let mut tts_tx_opt = Some(tts_tx); // wrap in Option so we can take() it

                let infer_result =
                    model.infer_audio_streaming(&resampled, &prompt, |event| match event {
                        TokenEvent::ResponseToken(text) => {
                            response_buf.push_str(&text); // Accumulate spoken response
                        }

                        TokenEvent::SentinelReached => {
                            // Optional: send partial response to TTS immediately
                            let partial = response_buf.trim().to_string();
                            if !partial.is_empty() {
                                if let Some(tx) = &tts_tx_opt {
                                    let _ = tx.send(partial);
                                }
                            }
                            emit(BabiloEvent::SentinelReached);
                        }
                        TokenEvent::AnalysisToken(text) => {
                            analysis_buf.push_str(&text); // Accumulate JSON analysis
                        }
                        TokenEvent::Done => {
                            drop(tts_tx_opt.take());

                            // Finalize response (trim, validate)
                            let final_response = response_buf.trim().to_string();
                            if final_response.is_empty() {
                                emit(BabiloEvent::Error {
                                    message: "Empty response from model".into(),
                                });
                                return;
                            }

                            // Build the complete analysis
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

                if let Err(e) = infer_result {
                    emit(BabiloEvent::Error {
                        message: e.to_string(),
                    });
                }
            }
        }

        let _ = tts_handle.join();
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn test_inference(
    test_prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut llm_lock = state.llm_engine.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut model) = *llm_lock {
        if model.model().context_is_full(model.state().n_past, 256) {
            let _ = model.reset();
        }
        model.infer_text(&test_prompt).map_err(|e| e.to_string())
    } else {
        Ok("No LLM model available".into())
    }
}

#[tauri::command]
pub fn reset_conversation(state: State<'_, AppState>) -> Result<bool, String> {
    let mut llm_lock = state.llm_engine.lock().map_err(|e| e.to_string())?;

    if let Some(ref mut model) = *llm_lock {
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
    let engine = state.llm_engine.lock().map_err(|e| e.to_string())?;

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

#[tauri::command]
pub async fn get_list_modes() -> Result<Vec<ModeFileInfo>, String> {
    crate::modes::list_modes().map_err(|e| e.to_string())
}
