/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Unified error handling with specific types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Audio: {0}")]
    Audio(#[from] AudioError),

    #[error("LLM: {0}")]
    Llm(#[from] LlmError),

    #[error("TTS: {0}")]
    Tts(#[from] TtsError),

    #[error("Configuración: {0}")]
    Config(String),

    #[error("Estado: {0}")]
    State(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tauri: {0}")]
    Tauri(String),

    #[error("Modo: {0}")]
    Mode(#[from] ModeError),

    #[error("Sesión: {0}")]
    Session(#[from] SessionError),
}

impl From<cpal::DevicesError> for AppError {
    fn from(e: cpal::DevicesError) -> Self {
        AppError::Audio(AudioError::DeviceConfig(e))
    }
}

impl From<cpal::BuildStreamError> for AppError {
    fn from(e: cpal::BuildStreamError) -> Self {
        AppError::Audio(AudioError::StreamBuild(e))
    }
}

impl From<cpal::PlayStreamError> for AppError {
    fn from(e: cpal::PlayStreamError) -> Self {
        AppError::Audio(AudioError::StreamPlay(e))
    }
}

impl From<cpal::PauseStreamError> for AppError {
    fn from(e: cpal::PauseStreamError) -> Self {
        AppError::Audio(AudioError::StreamPause(e))
    }
}

// ─────────────────────────────────────────────────────────────
// Domain-specific errors
// ─────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device configuration: {0}")]
    DeviceConfig(#[source] cpal::DevicesError),

    #[error("Stream build: {0}")]
    StreamBuild(#[source] cpal::BuildStreamError),

    #[error("Stream play: {0}")]
    StreamPlay(#[source] cpal::PlayStreamError),

    #[error("Stream pause: {0}")]
    StreamPause(#[source] cpal::PauseStreamError),

    #[error("Unsupported sample format")]
    UnsupportedSampleFormat,

    #[error("Empty audio buffer")]
    EmptyBuffer,

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Stream configuration: {0}")]
    StreamConfig(#[source] cpal::DefaultStreamConfigError), // ← new

    #[error("Capture already active")]
    CaptureAlreadyActive,
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Backend initialization: {0}")]
    BackendInit(String),

    #[error("Model loading: {0}")]
    ModelLoad(String),

    #[error("Context initialization: {0}")]
    ContextInit(String),

    #[error("MTMD initialization: {0}")]
    MtmdInit(String),

    #[error("Tokenization: {0}")]
    Tokenization(String),

    #[error("Decoding: {0}")]
    Decode(String),

    #[error("Sampling: {0}")]
    Sampling(String),

    #[error("Context full without reset capability")]
    ContextFull,

    #[error("Model not initialized")]
    NotInitialized,

    #[error("Analysis missing field {0}")]
    MissingField(String),
}

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("ONNX session loading: {0}")]
    SessionLoad(String),

    #[error("Inference execution: {0}")]
    Inference(String),

    #[error("Voice loading: {0}")]
    VoiceLoad(String),

    #[error("Empty text")]
    EmptyText,

    #[error("Audio generation: {0}")]
    AudioGeneration(String),

    #[error("TTS configuration not available")]
    ConfigMissing,

    #[error("ORT Tensor: {0}")]
    Tensor(String), // ← new, for Tensor::from_array
}

// ─────────────────────────────────────────────────────────────
// Mode Errors
// ─────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum ModeError {
    #[error("Modes directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Failed to read file '{path}': {source}")]
    IoRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid JSON in '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Mode not found: {0}")]
    NotFound(String),

    #[error("Duplicate mode with ID: {0}")]
    DuplicateId(String),

    #[error("Missing required field in mode '{path}': {field}")]
    MissingField { path: String, field: String },

    #[error("Mode validation failed: {0}")]
    Validation(String),
}

// ─────────────────────────────────────────────────────────────
// Session Errors
// ─────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session already active for this user")]
    AlreadyActive,

    #[error("Attempt to operate on non-existent session")]
    NotInitialized,

    #[error("Invalid state transition: {from} → {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Mode '{mode_id}' not available for session")]
    ModeUnavailable { mode_id: String },

    #[error("System prompt generation: {0}")]
    PromptComposition(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Operation not allowed in current state: {0}")]
    OperationNotAllowed(String),

    #[error("Session data load error: {0}")]
    LoadError(String),
}

// ─────────────────────────────────────────────────────────────
// Conversion to Tauri responses
// ─────────────────────────────────────────────────────────────

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<serde_json::Error> for TtsError {
    fn from(e: serde_json::Error) -> Self {
        TtsError::SessionLoad(e.to_string())
    }
}

impl From<cpal::DefaultStreamConfigError> for AppError {
    fn from(e: cpal::DefaultStreamConfigError) -> Self {
        AppError::Audio(AudioError::StreamConfig(e))
    }
}
