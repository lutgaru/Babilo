//! Comandos Tauri exponibles al frontend

use tauri::{ State};
use serde::{Serialize, Deserialize};
use crate::{
    state::AppState,
    errors::{AppError},
    audio::list_input_devices,
};

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
    let mut engine_lock = state.tts_engine.lock()
        .map_err(|e| e.to_string())?;

    match engine_lock.as_mut() {
        None => Err("TTS engine not initialized".into()),
        Some(engine) => engine
            .speak(&text, &voice_id, "en", 1.0, 30)
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn list_voices(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let engine_lock = state.tts_engine.lock()
        .map_err(|e| e.to_string())?;
    
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
    // 1. Limpiar captura anterior si existe
    {
        let mut capture_lock = state.audio_capture.lock()
            .map_err(|e| e.to_string())?;
        if let Some(ref mut capture) = *capture_lock {
            let _ = capture.stop(); // Ignoramos errores como en tu lógica original
        }
        *capture_lock = None;
    }

    // 2. Crear nueva instancia de AudioCapture (lógica de selección intacta)
    let mut capture = if let Some(ref name) = device_name {
        crate::audio::capture::AudioCapture::with_device_name(name)
    } else {
        crate::audio::capture::AudioCapture::default()
    }.map_err(|e| e.to_string())?;

    // 3. Guardar sample_rate en estado para referencia externa
    let sample_rate = capture.sample_rate();
    if let Ok(mut hz) = state.sample_rate.lock() {
        *hz = sample_rate;
    }

    // 4. Iniciar captura (aquí se ejecuta tu lógica crítica de stream)
    capture.start().map_err(|e| e.to_string())?;

    // 5. Guardar en estado
    let mut capture_lock = state.audio_capture.lock()
        .map_err(|e| e.to_string())?;
    *capture_lock = Some(capture);

    Ok(())
}

#[tauri::command]
pub async fn stop_and_process(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Detener captura y extraer audio (responsabilidad delegada a capture.rs)
    let (audio_raw, src_hz) = {
        let mut capture_lock = state.audio_capture.lock()
            .map_err(|e| e.to_string())?;
        
        let capture = capture_lock.as_mut()
            .ok_or("No active audio capture")?;
        
        capture.stop().map_err(|e| e.to_string())?;
        let buffer = capture.take_buffer();
        let hz = capture.sample_rate();
        (buffer, hz)
    };

    if audio_raw.is_empty() {
        return Err("Audio buffer empty".into());
    }

    // 2. Resampling (se mantiene en mod.rs porque es procesamiento, no captura)
    let target_hz = 16000.0;
    let resampled = if (src_hz as f32 - target_hz).abs() > f32::EPSILON {
        crate::audio::MelPreprocessor::resample(&audio_raw, src_hz as f32, target_hz)
    } else {
        audio_raw
    };

    // 3. Inferencia (orquestación, no cambia)
    let mut llm_lock = state.llm_engine.lock()
        .map_err(|e| e.to_string())?;
    
    if let Some(ref mut model) = *llm_lock {
        if model.model().context_is_full(model.state().n_past, 256) {
            let _ = model.reset();
        }
        model.infer_audio(&resampled, &prompt)
            .map_err(|e| e.to_string())
    } else {
        Ok(format!("[Echo] {}", prompt))
    }
}

#[tauri::command]
pub async fn test_inference(
    test_prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut llm_lock = state.llm_engine.lock()
        .map_err(|e| e.to_string())?;
    
    if let Some(ref mut model) = *llm_lock {
        if model.model().context_is_full(model.state().n_past, 256) {
            let _ = model.reset();
        }
        model.infer_text(&test_prompt)
            .map_err(|e| e.to_string())
    } else {
        Ok("No LLM model available".into())
    }
}

#[tauri::command]
pub fn reset_conversation(state: State<'_, AppState>) -> Result<bool, String> {
    let mut llm_lock = state.llm_engine.lock()
        .map_err(|e| e.to_string())?;

    if let Some(ref mut model) = *llm_lock {
        model.reset()
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
    let engine = state.llm_engine.lock()
        .map_err(|e| e.to_string())?;

    if let Some(ref engine) = *engine {
        let (used, total) = engine.model().context_usage(engine.state().n_past);
        Ok(ContextUsage {
            used,
            total,
            percent: if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 },
        })
    } else {
        Ok(ContextUsage { used: 0, total: 4096, percent: 0.0 })
    }
}