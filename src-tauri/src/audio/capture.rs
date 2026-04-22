//! Captura de audio desde hardware usando CPAL

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use crate::{config::AudioConfig, errors::{AppError, AudioError}};

pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
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

        let mut config: StreamConfig = device.default_input_config()?.into();
        config.sample_rate = 16000;
        config.channels = 1;

        Ok(Self {
            device,
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
        })
    }

    /// Crea una instancia con un dispositivo específico por nombre
    pub fn with_device_name(name: &str) -> Result<Self, AppError> {
        let host = cpal::default_host();
        let device = host
            .input_devices()
            .map_err(|e| AudioError::DeviceConfig(e))?
            .find(|d| d.name().ok().as_ref() == Some(&name.to_string()))
            .ok_or(AudioError::DeviceNotFound(name.into()))?;

        let mut config: StreamConfig = device.default_input_config()?.into();
        config.sample_rate = 16000;
        config.channels = 1;

        Ok(Self {
            device,
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
        })
    }

    /// Inicia la captura de audio
    pub fn start(&mut self) -> Result<Stream, AppError> {
        self.running.store(true, Ordering::SeqCst);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let n_channels = self.config.channels as usize;

        let err_fn = |err| eprintln!("❌ Error de audio: {}", err);

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
                            buf.push(frame[0]);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleFormat.into()),
        };

        stream.play().map_err(AudioError::StreamPlay)?;
        Ok(stream)
    }

    /// Detiene la captura
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Extrae y limpia el buffer de audio
    pub fn take_buffer(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// Obtiene la configuración actual
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Obtiene la frecuencia de muestreo actual
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }
}

/// Enumera los dispositivos de entrada disponibles
pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, AppError> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| AudioError::DeviceConfig(e))?
        .filter_map(|device| {
            device.name().ok().map(|name| AudioDeviceInfo {
                is_default: default_name.as_ref() == Some(&name),
                name,
            })
        })
        .collect();

    Ok(devices)
}