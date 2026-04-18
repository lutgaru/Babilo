// ── Imports ──────────────────────────────────────────────────────────────
use serde::{Deserialize, Serialize};
use std::sync::Mutex;  // ✅ Asegurar que Mutex está importado
use tauri::{State, Manager};  // ✅ Agregar Manager aquí


// Tus módulos
mod tts;  // ← Tu módulo TTS existente
mod audio;
mod llama;

use audio::{AudioCapture, AudioConfig, MelPreprocessor};
use llama::AudioLLM;
use tts::TtsEngine;  // ← Ajusta según tu estructura real de tts.rs

// ── AppState GLOBAL ─────────────────────────────────────────────────────
pub struct AppState {
    pub tts_engine: Mutex<Option<TtsEngine>>,
    pub audio_capture: Mutex<Option<AudioCapture>>,
    pub audio_llm: Mutex<Option<AudioLLM>>,
    pub preprocessor: MelPreprocessor,
}

// ── Structs de respuesta ────────────────────────────────────────────────
#[derive(Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool,
    pub message: String,
    pub audio_path: Option<String>,
}

// ── Comandos Tauri ──────────────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn synthesize(
    text: String,
    voice: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<i8>, String> {
    let voice_id = voice.unwrap_or_else(|| "F1".to_string());
    // ✅ Desencadenar Arc<Mutex<>> correctamente
    let mut engine_lock = state.tts_engine.lock().map_err(|e| e.to_string())?;
    const DENOISING_STEPS: usize = 30;
    
    match engine_lock.as_mut() {
        None => Err("Engine no inicializado".into()),
        Some(engine) => engine
            .speak(&text, &voice_id, "en", 1.0, DENOISING_STEPS)
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
fn list_voices(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut engine_lock = state.tts_engine.lock().map_err(|e| e.to_string())?;
    match engine_lock.as_ref() {
        Some(engine) => Ok(engine.list_voices()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
async fn start_listening(state: State<'_, AppState>) -> Result<(), String> {
    let mut capture_lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut cap) = *capture_lock {
        cap.start().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Audio capture not initialized".into())
    }
}

#[tauri::command]
async fn stop_and_process(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Detener captura
    {
        let capture_lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
        if let Some(ref cap) = *capture_lock {
            cap.stop();
        }
    }
    
    // Obtener audio
    let audio = {
        let capture_lock = state.audio_capture.lock().map_err(|e| e.to_string())?;
        if let Some(ref cap) = *capture_lock {
            cap.take_buffer()
        } else {
            return Err("No audio captured".into());
        }
    };
    
    // Preprocesar
    let config = AudioConfig {
        sample_rate: 16000,
        channels: 1,
        chunk_duration_secs: 30,
        mel_bins: 128,
        window_size: 320,
        hop_size: 160,
    };
    let mel_chunks = state.preprocessor.process(&audio, &config);
    
    // Inferir (placeholder hasta que llama.cpp soporte Gemma 4 audio nativo)
    let mut llm_lock = state.audio_llm.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut model) = *llm_lock {
        model.infer(mel_chunks, &prompt)
            .map_err(|e| format!("Inference error: {}", e))
    } else {
        // 🔧 Fallback: retornar el prompt como eco si no hay LLM
        Ok(format!("[Echo] {}", prompt))
    }
}

// ── Entry Point ─────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            synthesize, 
            list_voices,
            start_listening,
            stop_and_process,
        ])
        .setup(|app| {
            let preprocessor = MelPreprocessor::new(16000, 128, 512);
            
            // ✅ AudioLLM puede fallar, es opcional
            let model_path = AudioLLM::models_dir().join("gemma-4-E4B-it-Q4_0.gguf");
            let audio_llm = AudioLLM::new(&model_path).ok();
            
            let audio_capture = AudioCapture::new().ok();
            
            // ✅ Usar app.handle() en Tauri 2 (no app_handle)
            let engine = match TtsEngine::new(tts::assets_dir(), app.handle().clone()) {
                Ok(e) => {
                    eprintln!("✅ TTS Engine inicializado");
                    Some(e)
                }
                Err(e) => {
                    eprintln!("❌ Error TTS: {}", e);
                    None
                }
            };
            
            // ✅ Usar Mutex directamente en el estado compartido
            app.manage(AppState {
                tts_engine: Mutex::new(engine),
                audio_capture: Mutex::new(audio_capture),
                audio_llm: Mutex::new(audio_llm),
                preprocessor,
            });
            
            // ✅ Usar get_webview_window desde Manager
            if let Some(window) = app.get_webview_window("main") {
                println!("🪟 Ventana: {:?}", window.url());
                window.show().unwrap();
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}