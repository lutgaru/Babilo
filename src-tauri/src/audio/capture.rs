//! Captura de audio desde hardware usando CPAL

use crate::errors::{AppError, AudioError};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>, // ← Nuevo: almacenamos el stream internamente
}

pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

impl AudioCapture {
    /// Crea una nueva instancia con el dispositivo de entrada por defecto
    pub fn default() -> Result<Self, AppError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound("default input".into()))?;

        let config: StreamConfig = device.default_input_config()?.into();
        // ⚠️ No forzamos sample_rate aquí, usamos el nativo del dispositivo

        Ok(Self {
            device,
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            stream: None,
        })
    }

    /// Crea una instancia con un dispositivo específico por nombre
    pub fn with_device_name(name: &str) -> Result<Self, AppError> {
        let host = cpal::default_host();
        let device = host
            .input_devices()
            .map_err(|e| AudioError::DeviceConfig(e))?
            .find(|d| {
                d.description()
                    .map(|desc| desc.name() == name)
                    .unwrap_or(false)
            })
            .ok_or(AudioError::DeviceNotFound(name.into()))?;

        let config: StreamConfig = device.default_input_config()?.into();

        Ok(Self {
            device,
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            stream: None,
        })
    }

    /// Inicia la captura de audio (lógica equivalente a start_listening original)
    pub fn start(&mut self) -> Result<(), AppError> {
        if self.stream.is_some() {
            return Err(AudioError::CaptureAlreadyActive.into());
        }

        self.running.store(true, Ordering::SeqCst);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let n_channels = self.config.channels as usize;

        let err_fn = |err| eprintln!("❌ Stream error: {}", err);

        // ← Esta es la lógica crítica que se mantiene intacta
        let stream = match self.device.default_input_config()?.sample_format() {
            SampleFormat::I16 => self.device.build_input_stream(
                &self.config,
                move |data: &[i16], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        for frame in data.chunks(n_channels) {
                            buf.push(frame[0] as f32 / 32768.0);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::F32 => self.device.build_input_stream(
                &self.config,
                move |data: &[f32], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        for frame in data.chunks(n_channels) {
                            buf.push(frame[0]); // ← Mantiene exactamente tu lógica original
                        }
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleFormat.into()),
        };

        stream.play().map_err(AudioError::StreamPlay)?;
        self.stream = Some(stream); // ← Guardamos para mantenerlo vivo
        Ok(())
    }

    /// Detiene la captura y pausa el stream
    pub fn stop(&mut self) -> Result<(), AppError> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause(); // Silenciamos errores de pause, como en tu código original
        }
        Ok(())
    }

    /// Extrae y limpia el buffer de audio (thread-safe)
    pub fn take_buffer(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// Obtiene la frecuencia de muestreo del dispositivo
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate // SampleRate es newtype(u32)
    }

    /// Obtiene la configuración actual (útil para debugging)
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Verifica si la captura está activa
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Enumera los dispositivos de entrada disponibles
pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, AppError> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    let devices = host
        .input_devices()
        .map_err(|e| AudioError::DeviceConfig(e))?
        .filter_map(|device| {
            device.description().ok().map(|desc| AudioDeviceInfo {
                name: desc.name().to_string(),
                is_default: desc.name() == default_name.as_deref().unwrap_or(""),
            })
        })
        .collect();

    Ok(devices)
}