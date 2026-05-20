/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Main library - Tauri configuration and setup only

use crate::{session::SessionManager, state::AppState};
use tauri::{Builder, Emitter, Manager};

// Modules
pub mod audio;
pub mod commands;
pub mod config;
pub mod errors;
pub mod llama;
pub mod modes;
pub mod schemas;
pub mod session;
pub mod state;
pub mod tts;
pub mod utils;

// Re-export commands for the macro
pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::list_voices,
            commands::list_audio_devices,
            commands::start_listening,
            commands::stop_and_process_streaming,
            commands::reset_conversation,
            commands::get_context_usage,
            commands::start_session,
            commands::end_session,
            commands::get_list_modes,
            commands::process_text_streaming
        ])
        .setup(|app| {
            // 1. Ultra-lightweight and instant initializations first
            utils::init_logging();

            // Set up global state with empty/initial containers
            app.manage(AppState {
                config: config::AppConfig::default(),
                audio_capture: std::sync::Mutex::new(None),
                audio_stream: std::sync::Mutex::new(None),
                audio_buffer: std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(
                    16000 * 30,
                ))),
                sample_rate: std::sync::Mutex::new(16000),
                preprocessor: audio::MelPreprocessor::new(16000, 128, 512),
                session_manager: std::sync::Arc::new(std::sync::Mutex::new(SessionManager::new())),
            });

            // Clone app_handle to move safely between threads
            let app_handle = app.handle().clone();

            // 2. SPARK THE ASYNC THREAD: Heavy lifting happens down here
            tauri::async_runtime::spawn(async move {
                // Model paths
                let models_dir = utils::models_dir();
                let model_path = models_dir.join("gemma-4-E4B-it-Q4_0.gguf");
                let mmproj_path = models_dir.join("mmproj-BF16.gguf");

                // Initialize LLM
                let llm_engine = if model_path.exists() {
                    let mmproj = if mmproj_path.exists() {
                        eprintln!("✅ mmproj found, enabling multimodal support");
                        Some(mmproj_path.as_path())
                    } else {
                        eprintln!("⚠️ mmproj not found, text-only mode");
                        None
                    };

                    match llama::LlmModel::new(&model_path, mmproj, config::LlmConfig::default()) {
                        Ok(model) => Some(llama::InferenceEngine::new(model)),
                        Err(e) => {
                            eprintln!("❌ Failed to load LLM: {}", e);
                            let _ = app_handle
                                .emit("babilo://core-error", format!("Failed to load LLM: {}", e));
                            None
                        }
                    }
                } else {
                    eprintln!("❌ Model file not found: {:?}", model_path);
                    let _ = app_handle.emit(
                        "babilo://core-error",
                        "Gemma model file not found".to_string(),
                    );
                    None
                };

                // Initialize TTS
                let tts_engine = match tts::TtsEngine::new(tts::assets_dir(), app_handle.clone()) {
                    Ok(e) => {
                        eprintln!("✅ TTS Engine initialized");
                        Some(e)
                    }
                    Err(e) => {
                        eprintln!("❌ TTS init error: {}", e);
                        let _ = app_handle
                            .emit("babilo://core-error", format!("TTS init error: {}", e));
                        None
                    }
                };

                // Inject engines into global State Manager once ready
                {
                    let state = app_handle.state::<AppState>();
                    let mut manager = state.session_manager.lock().unwrap();
                    manager.load_engines(llm_engine, tts_engine);
                }

                // 3. READY! Notify the frontend (Lit) to unmount the Splash
                let _ = app_handle.emit("babilo://core-ready", ());
            });

            // Return OK immediately. Window opens rendering initial HTML.
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
