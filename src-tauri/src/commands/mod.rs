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
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    
    // Seleccionar dispositivo
    let device = if let Some(ref name) = device_name {
        host.input_devices()
            .map_err(|e| format!("Error enumerating devices: {}", e))?
            .find(|d| {
                d.description()
                    .map(|desc| desc.name() == name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Microphone not found: '{}'", name))?
    } else {
        host.default_input_device()
            .ok_or("No default input device found")?
    };

    let config = device.default_input_config()
        .map_err(|e| e.to_string())?;
    let actual_hz = config.sample_rate();

    // Guardar frecuencia
    if let Ok(mut hz) = state.sample_rate.lock() {
        *hz = actual_hz;
    }

    state.clear_audio_buffer();
    let audio_buffer = state.audio_buffer();

    let n_channels = config.channels() as usize;
    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buffer) = audio_buffer.lock() {
                    for frame in data.chunks(n_channels) {
                        buffer.push(frame[0]);
                    }
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    // Guardar handle al stream
    let mut stream_lock = state.audio_stream.lock()
        .map_err(|e| e.to_string())?;
    *stream_lock = Some(crate::state::AudioStreamHandle::new(stream));

    Ok(())
}

#[tauri::command]
pub async fn stop_and_process(
    prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Detener stream
    {
        let mut stream_lock = state.audio_stream.lock()
            .map_err(|e| e.to_string())?;
        if let Some(handle) = stream_lock.take() {
            let _ = handle.pause();
        }
    }

    // 2. Obtener audio
    let (audio_raw, src_hz) = {
        let buffer = state.audio_buffer.lock()
            .map_err(|e| e.to_string())?
            .clone();
        let hz = *state.sample_rate.lock()
            .map_err(|e| e.to_string())?;
        (buffer, hz as f32)
    };

    if audio_raw.is_empty() {
        return Err("Audio buffer empty".into());
    }

    // 3. Resampling si es necesario
    let target_hz = 16000.0;
    let resampled = if (src_hz - target_hz).abs() > f32::EPSILON {
        crate::audio::MelPreprocessor::resample(&audio_raw, src_hz, target_hz)
    } else {
        audio_raw
    };

    // 4. Inferencia
    let mut llm_lock = state.llm_engine.lock()
        .map_err(|e| e.to_string())?;
    
    if let Some(ref mut model) = *llm_lock {
        // Reset si contexto lleno
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