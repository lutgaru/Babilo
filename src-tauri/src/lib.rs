/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Biblioteca principal - solo configuración y setup de Tauri

use tauri::{Manager, Builder};
use crate::state::AppState;

// Módulos
pub mod audio;
pub mod commands;
pub mod config;
pub mod errors;
pub mod llama;
pub mod state;
pub mod tts;
pub mod utils;
pub mod schemas; 

// Re-export de commands para el macro
pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::synthesize,
            commands::list_voices,
            commands::list_audio_devices,
            commands::start_listening,
            commands::stop_and_process,
            commands::stop_and_process_streaming,
            commands::test_inference,
            commands::reset_conversation,
            commands::get_context_usage,
        ])
        .setup(|app| {
            // Inicializar logging
            utils::init_logging();

            // Rutas a modelos
            let models_dir = utils::models_dir();
            let model_path = models_dir.join("gemma-4-E4B-it-Q4_0.gguf");
            let mmproj_path = models_dir.join("mmproj-BF16.gguf");

            // Inicializar LLM
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
                        None
                    }
                }
            } else {
                eprintln!("❌ Model file not found: {:?}", model_path);
                None
            };

            // Inicializar TTS
            let tts_engine = match tts::TtsEngine::new(
                tts::assets_dir(),
                app.handle().clone()
            ) {
                Ok(e) => {
                    eprintln!("✅ TTS Engine initialized");
                    Some(e)
                }
                Err(e) => {
                    eprintln!("❌ TTS init error: {}", e);
                    None
                }
            };

            // Configurar estado global
            app.manage(AppState {
                config: config::AppConfig::default(),
                tts_engine: std::sync::Arc::new(std::sync::Mutex::new(tts_engine)),
                llm_engine: std::sync::Arc::new(std::sync::Mutex::new(llm_engine)),
                audio_capture: std::sync::Mutex::new(None),
                audio_stream: std::sync::Mutex::new(None),
                audio_buffer: std::sync::Arc::new(std::sync::Mutex::new(
                    Vec::with_capacity(16000 * 30)
                )),
                sample_rate: std::sync::Mutex::new(16000),
                preprocessor: audio::MelPreprocessor::new(16000, 128, 512),
            });

            // Mostrar ventana principal
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}