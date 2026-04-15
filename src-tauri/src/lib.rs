mod tts;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::http::Response;
use tauri::Manager;
use tts::TtsEngine;
// Estado global del engine (se inicializa una vez)
pub struct AppState {
    engine: Mutex<Option<TtsEngine>>,
}

#[derive(Serialize, Deserialize)]
pub struct TtsResult {
    pub success: bool,
    pub message: String,
    pub audio_path: Option<String>,
}

// ── Comandos ──────────────────────────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn synthesize(
    text: String,
    voice: Option<String>,
    state: tauri::State<AppState>,
) -> Result<Vec<i8>, String> {
    // ← Retornar Vec<i8>
    let voice_id = voice.unwrap_or_else(|| "F1".to_string());
    let mut engine_lock = state.engine.lock().map_err(|e| e.to_string())?;
    const DENOISING_STEPS: usize = 30;
    match engine_lock.as_mut() {
        None => Err("Engine no inicializado".into()),
        Some(engine) => engine
            .speak(&text, &voice_id, "en", 1.0, DENOISING_STEPS)
            .map_err(|e| e.to_string()), // ← Propagar error como String
    }
}

#[tauri::command]
fn list_voices(state: tauri::State<AppState>) -> Vec<String> {
    let engine_lock = state.engine.lock().unwrap();
    match engine_lock.as_ref() {
        Some(engine) => engine.list_voices(),
        None => vec![],
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Intentar inicializar el engine al arrancar

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // .manage(AppState {
        //     engine: Mutex::new(engine),
        // })
        .invoke_handler(tauri::generate_handler![greet, synthesize, list_voices,])
        .setup(|app| {
            let engine = match TtsEngine::new(tts::assets_dir(), app.app_handle().clone()) {
                Ok(e) => {
                    eprintln!("✅ Engine inicializado correctamente");
                    Some(e)
                }
                Err(e) => {
                    eprintln!("❌ Error al inicializar engine: {}", e);
                    // Imprimir más detalles si es un error de archivo
                    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                        eprintln!("   Detalle IO: {:?}", io_err);
                    }
                    None
                }
            };
            app.manage(AppState {
                engine: Mutex::new(engine),
            });
            let window = app.get_webview_window("main").unwrap();
            println!("Ventana creada en: {:?}", window.url());
            window.show().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
