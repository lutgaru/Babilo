/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
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
    sample_format: SampleFormat, // ← Cacheado para no llamar default_input_config() dos veces
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>,
}

pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

impl AudioCapture {
    fn from_device(device: Device) -> Result<Self, AppError> {
        let supported = device.default_input_config()?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();

        Ok(Self {
            device,
            config,
            sample_format,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            stream: None,
        })
    }

    pub fn default() -> Result<Self, AppError> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound("default input".into()))?;
        Self::from_device(device)
    }

    pub fn with_device_name(name: &str) -> Result<Self, AppError> {
        let device = cpal::default_host()
            .input_devices()
            .map_err(|e| AudioError::DeviceConfig(e))?
            .find(|d| {
                d.description()
                    .map(|desc| desc.name() == name)
                    .unwrap_or(false)
            })
            .ok_or(AudioError::DeviceNotFound(name.into()))?;
        Self::from_device(device)
    }

    pub fn start(&mut self) -> Result<(), AppError> {
        if self.stream.is_some() {
            return Err(AudioError::CaptureAlreadyActive.into());
        }

        self.running.store(true, Ordering::SeqCst);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let n_channels = self.config.channels as usize;
        let err_fn = |err| eprintln!("❌ Stream error: {}", err);

        // ← sample_format ya cacheado, sin segunda llamada a default_input_config()
        let stream = match self.sample_format {
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
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), AppError> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(stream) = self.stream.take() {
            let _ = stream.pause();
        }
        Ok(())
    }

    pub fn take_buffer(&self) -> Vec<f32> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }

    /// Obtiene la frecuencia de muestreo del dispositivo
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate // SampleRate es newtype(u32)
    }

    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, AppError> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

    Ok(host
        .input_devices()
        .map_err(AudioError::DeviceConfig)?
        .filter_map(|device| {
            device.description().ok().map(|desc| AudioDeviceInfo {
                name: desc.name().to_string(),
                is_default: desc.name() == default_name.as_deref().unwrap_or(""),
            })
        })
        .collect())
}
