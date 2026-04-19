// ── Imports ──────────────────────────────────────────────────────────────
use audio::{AudioCapture, MelPreprocessor};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
// Tus módulos
mod audio;
mod llama;
mod tts;

use llama::AudioLLM;
use tts::TtsEngine;
// ── Wrapper seguro para cpal::Stream ─────────────────────────────────────
// cpal::Stream contiene *mut () que no es Send por defecto, pero CPAL garantiza
// que es seguro moverlo entre threads para control (pause/play/drop).
// ── Wrapper seguro para cpal::Stream ─────────────────────────────────────
pub struct SafeStream(Option<cpal::Stream>);

unsafe impl Send for SafeStream {}
unsafe impl Sync for SafeStream {}

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub id: String, // Optional: use for precise selection
}

#[tauri::command]
fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();

    let devices = host
        .input_devices()
        .map_err(|e| format!("Error enumerating devices: {}", e))?
        .filter_map(|device| {
            device.name().ok().map(|name| AudioDevice {
                name,
                id: String::new(), // CPAL doesn't expose stable IDs; name is usually enough
            })
        })
        .collect();

    Ok(devices)
}

impl SafeStream {
    pub fn new(stream: cpal::Stream) -> Self {
        Self(Some(stream))
    }

    // ✅ PauseStreamError para pause()
    pub fn pause(&self) -> Result<(), cpal::PauseStreamError> {
        if let Some(ref stream) = self.0 {
            stream.pause()
        } else {
            Ok(())
        }
    }

    // ✅ PlayStreamError para play()
    pub fn play(&self) -> Result<(), cpal::PlayStreamError> {
        if let Some(ref stream) = self.0 {
            stream.play()
        } else {
            Ok(())
        }
    }

    pub fn take(&mut self) -> Option<cpal::Stream> {
        self.0.take()
    }
}
// ── AppState GLOBAL ─────────────────────────────────────────────────────
pub struct AppState {
    pub tts_engine: Mutex<Option<TtsEngine>>,
    pub audio_capture: Mutex<Option<AudioCapture>>, // Mantenemos por compatibilidad con tu módulo
    pub audio_llm: Mutex<Option<AudioLLM>>,
    pub preprocessor: MelPreprocessor,

    // Buffer compartido para los samples de audio grabados
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,

    // ✅ NUEVO: Referencia al stream de CPAL para poder pausarlo/detenerlo
    pub audio_stream: Mutex<Option<SafeStream>>,
    pub current_sample_rate: Mutex<u32>,
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

// ✅ CORREGIDO: Sin 'pub', guarda el stream en AppState en lugar de olvidar
#[tauri::command]
fn start_listening(
    device_name: Option<String>, // ✅ Nuevo parámetro opcional
    state: State<'_, AppState>,
) -> Result<(), String> {
    let host = cpal::default_host();

    // 🔍 Seleccionar dispositivo: por nombre o default
    let device = if let Some(ref name) = device_name {
        host.input_devices()
            .map_err(|e| format!("Error enumerating devices: {}", e))?
            .find(|d| d.name().ok().as_ref() == Some(name))
            .ok_or_else(|| format!("No se encontró el micrófono: '{}'", name))?
    } else {
        host.default_input_device()
            .ok_or("No se encontró micrófono por defecto")?
    };

    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let actual_hz = config.sample_rate().0;

    // Guardamos la frecuencia real en el estado
    if let Ok(mut hz_lock) = state.current_sample_rate.lock() {
        *hz_lock = actual_hz;
    }
    eprintln!("🎙️ Micro configurado a: {} Hz", actual_hz);

    let audio_buffer = Arc::clone(&state.audio_buffer);

    if let Ok(mut buffer) = audio_buffer.lock() {
        buffer.clear();
    } else {
        return Err("No se pudo acceder al buffer de audio".into());
    }

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buffer) = audio_buffer.lock() {
                    buffer.extend_from_slice(data);
                }
            },
            |err| eprintln!("Error en el stream: {}", err),
            None,
        )
        .map_err(|e| e.to_string())?;

    let safe_stream = SafeStream::new(stream);
    safe_stream.play().map_err(|e| e.to_string())?;

    let mut stream_lock = state.audio_stream.lock().map_err(|e| e.to_string())?;
    *stream_lock = Some(safe_stream);

    Ok(())
}

#[tauri::command]
async fn stop_and_process(prompt: String, state: State<'_, AppState>) -> Result<String, String> {
    // ── 1. Detener el stream ──────────────────────────────────────────────
    {
        let mut stream_lock = state.audio_stream.lock().map_err(|e| e.to_string())?;
        if let Some(safe_stream) = stream_lock.take() {
            let _ = safe_stream.pause();
        }
    }

    // ── 2. Obtener audio ──────────────────────────────────────────────────
    let (audio_raw, src_hz) = {
        let buffer = state
            .audio_buffer
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let hz = *state
            .current_sample_rate
            .lock()
            .map_err(|e| e.to_string())?;
        (buffer, hz as f32)
    };

    println!(
        "🎙️ Audio capturado: {} samples ({:.1}s a 16kHz)",
        audio_raw.len(),
        audio_raw.len() as f32 / 16000.0
    );

    if audio_raw.is_empty() {
        return Err("Audio buffer vacío — ¿se presionó el botón antes de hablar?".into());
    }

    let target_hz = 16000.0;

    // ── 2.5 Resampling inteligente ──────────────────────────────────────
    let resampled_audio = if src_hz != target_hz {
        let ratio = src_hz / target_hz;
        eprintln!(
            "🔄 Resampling dinámico: {}Hz -> {}Hz (Ratio: {:.2})",
            src_hz, target_hz, ratio
        );

        let mut resampled = Vec::with_capacity((audio_raw.len() as f32 / ratio) as usize);
        let mut i = 0.0;
        while i < audio_raw.len() as f32 {
            resampled.push(audio_raw[i as usize]);
            i += ratio;
        }
        resampled
    } else {
        audio_raw
    };

    let mut llm_lock = state.audio_llm.lock().map_err(|e| e.to_string())?;
    let Some(ref mut model) = *llm_lock else {
        return Ok(format!("[Echo - no LLM] {}", prompt));
    };

    // ── 3. Intentar infer_audio si existe el mmproj ───────────────────────
    let mmproj_path = AudioLLM::models_dir().join("mmproj-BF16.gguf");

    if mmproj_path.exists() {
        println!("✅ mmproj encontrado, usando infer_audio");

        model
            .infer_audio(&resampled_audio, &prompt) // ← solo 2 parámetros ahora
            .map_err(|e| format!("infer_audio error: {}", e))
    } else {
        // ── Fallback: usar infer() de texto con el prompt del usuario ─────
        println!(
            "⚠️ mmproj no encontrado en {:?}, usando fallback de texto",
            mmproj_path
        );
        Ok(format!("[Echo - no mmproj] "))
    }
}

#[tauri::command]
async fn test_inference(state: State<'_, AppState>, test_prompt: String) -> Result<String, String> {
    let mut llm_lock = state.audio_llm.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut model) = *llm_lock {
        model
            .infer(vec![], test_prompt.as_str())
            .map_err(|e| format!("Inference error: {}", e))
    } else {
        Ok("No LLM model available".into())
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
            list_audio_devices,
            start_listening,
            stop_and_process,
            test_inference,
        ])
        .setup(|app| {
            let preprocessor = MelPreprocessor::new(16000, 128, 512);

            let model_path = AudioLLM::models_dir().join("gemma-4-E4B-it-Q4_0.gguf");
            let mmproj_path = AudioLLM::models_dir().join("mmproj-BF16.gguf");

            // ↓ Pasar mmproj_path al constructor
            let audio_llm = if mmproj_path.exists() {
                AudioLLM::new(&model_path, Some(&mmproj_path)).ok()
            } else {
                eprintln!("⚠️ mmproj no encontrado, AudioLLM sin soporte multimodal");
                AudioLLM::new(&model_path, None).ok()
            };

            let audio_capture = AudioCapture::new().ok();

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

            // ✅ Inicializamos AppState con el nuevo campo audio_stream
            app.manage(AppState {
                tts_engine: Mutex::new(engine),
                audio_capture: Mutex::new(audio_capture),
                audio_llm: Mutex::new(audio_llm),
                preprocessor,
                audio_buffer: Arc::new(Mutex::new(Vec::new())),
                audio_stream: Mutex::new(None), // ✅ SafeStream envuelto en Mutex simple
                current_sample_rate: Mutex::new(16000), // Valor por defecto, se actualizará al iniciar la grabación
            });

            if let Some(window) = app.get_webview_window("main") {
                println!("🪟 Ventana: {:?}", window.url());
                window.show().unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
