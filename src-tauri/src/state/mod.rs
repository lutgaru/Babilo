/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Global application state management

use cpal::traits::StreamTrait;
use std::sync::{Arc, Mutex};

use crate::{
    audio::{AudioCapture, MelPreprocessor},
    config::AppConfig,
    session::SessionManager,
};

/// Safe wrapper for cpal::Stream (not Send by default)
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

/// Public handle to control the audio stream from commands
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

/// Global state accessible from Tauri commands
pub struct AppState {
    /// Application configuration
    pub config: AppConfig,

    /// Audio capture (hardware)
    pub audio_capture: Mutex<Option<AudioCapture>>,

    /// Handle to the active CPAL stream
    pub audio_stream: Mutex<Option<AudioStreamHandle>>,

    /// Shared buffer for audio samples
    pub audio_buffer: Arc<Mutex<Vec<f32>>>,

    /// Current device sample rate
    pub sample_rate: Mutex<u32>,

    /// Features preprocessor (Mel/FFT)
    pub preprocessor: MelPreprocessor,

    /// Session manager handling session lifecycle and state
    pub session_manager: Arc<Mutex<SessionManager>>,
}

impl AppState {
    /// Constructor with default configuration
    pub fn new() -> Self {
        let config = AppConfig::default();
        let preprocessor = MelPreprocessor::new(
            config.audio.sample_rate,
            config.audio.mel_bins,
            512, // FFT size
        );

        Self {
            config,
            audio_capture: Mutex::new(None),
            audio_stream: Mutex::new(None),
            audio_buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
            sample_rate: Mutex::new(16000),
            preprocessor,
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
        }
    }

    /// Checks if there is an active audio stream
    pub fn is_listening(&self) -> bool {
        self.audio_stream
            .lock()
            .map(|g| g.is_some())
            .unwrap_or(false)
    }

    /// Gets a reference to the audio buffer
    pub fn audio_buffer(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.audio_buffer)
    }

    /// Clears the audio buffer
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