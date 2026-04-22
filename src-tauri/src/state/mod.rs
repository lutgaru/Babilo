//! Gestión del estado global de la aplicación

use std::sync::{Arc, Mutex};
use cpal::traits::StreamTrait;

use crate::{
    audio::{AudioCapture, MelPreprocessor},
    llama::InferenceEngine,
    tts::TtsEngine,
    config::AppConfig,
};

/// Wrapper seguro para cpal::Stream (no Send por defecto)
pub struct SafeStream(Option<cpal::Stream>);

unsafe impl Send for SafeStream {}
unsafe impl Sync for SafeStream {}

impl SafeStream {
    pub fn new(stream: cpal::Stream) -> Self {
        Self(Some(stream))
    }

    pub fn pause(&self) -> Result<(), cpal::PauseStreamError> {
        self.0.as_ref().map_or(Ok(()), |s| s.pause())
    }

    pub fn play(&self) -> Result<(), cpal::PlayStreamError> {
        self.0.as_ref().map_or(Ok(()), |s| s.play())
    }

    pub fn take(&mut self) -> Option<cpal::Stream> {
        self.0.take()
    }
}

/// Handle público para controlar el stream de audio desde commands
#[derive(Clone)]
pub struct AudioStreamHandle {
    inner: Arc<Mutex<Option<SafeStream>>>,
}

impl AudioStreamHandle {
    pub fn new(stream: cpal::Stream) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(SafeStream::new(stream)))),
        }
    }

    pub fn pause(&self) -> Result<(), String> {
        self.inner
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|s| s.pause().map_err(|e| e.to_string()))
            .transpose()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if let Some(stream) = guard.take() {
            let _ = stream.pause();
        }
        Ok(())
    }
}

/// Estado global accesible desde los commands de Tauri
pub struct AppState {
    /// Configuración de la aplicación
    pub config: AppConfig,
    
    /// Motor TTS (ONNX)
    pub tts_engine: Mutex<Option<TtsEngine>>,
    
    /// Motor LLM (llama.cpp)
    pub llm_engine: Mutex<Option<InferenceEngine>>,
    
    /// Capturador de audio (hardware)
    pub audio_capture: Mutex<Option<AudioCapture>>,
    
    /// Handle al stream activo de CPAL
    pub audio_stream: Mutex<Option<AudioStreamHandle>>,
    
    /// Buffer compartido para samples de audio
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,
    
    /// Frecuencia de muestreo actual del dispositivo
    pub sample_rate: Mutex<u32>,
    
    /// Preprocesador de features (Mel/FFT)
    pub preprocessor: MelPreprocessor,
}

impl AppState {
    /// Constructor con configuración por defecto
    pub fn new() -> Self {
        let config = AppConfig::default();
        let preprocessor = MelPreprocessor::new(
            config.audio.sample_rate,
            config.audio.mel_bins,
            512, // FFT size
        );

        Self {
            config,
            tts_engine: Mutex::new(None),
            llm_engine: Mutex::new(None),
            audio_capture: Mutex::new(None),
            audio_stream: Mutex::new(None),
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            sample_rate: Mutex::new(16000),
            preprocessor,
        }
    }

    /// Verifica si hay un stream de audio activo
    pub fn is_listening(&self) -> bool {
        self.audio_stream
            .lock()
            .map(|g| g.is_some())
            .unwrap_or(false)
    }

    /// Obtiene una referencia al buffer de audio
    pub fn audio_buffer(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.audio_buffer)
    }

    /// Limpia el buffer de audio
    pub fn clear_audio_buffer(&self) {
        if let Ok(mut buf) = self.audio_buffer.lock() {
            buf.clear();
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}