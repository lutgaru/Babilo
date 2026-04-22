//! Módulo de audio: captura y procesamiento

pub mod capture;
pub mod processor;

pub use capture::{AudioCapture, AudioDeviceInfo, list_input_devices};
pub use processor::MelPreprocessor;

// Re-export del handle seguro para streams
// pub use crate::state::AudioStreamHandle;