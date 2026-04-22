//! Módulo TTS: síntesis de voz con ONNX
pub mod engine;
pub mod utils;
use std::path::PathBuf;

pub use engine::TtsEngine;
pub use utils::{UnicodeProcessor, VoiceStyle, load_voice_style};


/// Obtener ruta al directorio de assets
pub fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Raíz del proyecto no encontrada")
        .join("assets")
}