/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Manejo de errores unificado con tipos específicos

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
// Errores específicos por dominio
// ─────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Dispositivo no encontrado: {0}")]
    DeviceNotFound(String),

    #[error("Configuración de dispositivo: {0}")]
    DeviceConfig(#[source] cpal::DevicesError),

    #[error("Construcción de stream: {0}")]
    StreamBuild(#[source] cpal::BuildStreamError),

    #[error("Reproducción de stream: {0}")]
    StreamPlay(#[source] cpal::PlayStreamError),

    #[error("Pausa de stream: {0}")]
    StreamPause(#[source] cpal::PauseStreamError),

    #[error("Formato de muestra no soportado")]
    UnsupportedSampleFormat,

    #[error("Buffer de audio vacío")]
    EmptyBuffer,

    #[error("Error de procesamiento: {0}")]
    Processing(String),

    #[error("Configuración de stream: {0}")]
    StreamConfig(#[source] cpal::DefaultStreamConfigError), // ← nueva

    #[error("Captura ya activa")]
    CaptureAlreadyActive,
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Inicialización del backend: {0}")]
    BackendInit(String),

    #[error("Carga del modelo: {0}")]
    ModelLoad(String),

    #[error("Inicialización del contexto: {0}")]
    ContextInit(String),

    #[error("Inicialización MTMD: {0}")]
    MtmdInit(String),

    #[error("Tokenización: {0}")]
    Tokenization(String),

    #[error("Decodificación: {0}")]
    Decode(String),

    #[error("Muestreo: {0}")]
    Sampling(String),

    #[error("Contexto lleno y sin capacidad de reset")]
    ContextFull,

    #[error("Modelo no inicializado")]
    NotInitialized,

    #[error("Analysis missing field {0}")]
    MissingField(String),
}

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("Carga de sesión ONNX: {0}")]
    SessionLoad(String),

    #[error("Ejecución de inferencia: {0}")]
    Inference(String),

    #[error("Carga de voz: {0}")]
    VoiceLoad(String),

    #[error("Texto vacío")]
    EmptyText,

    #[error("Generación de audio: {0}")]
    AudioGeneration(String),

    #[error("Configuración TTS no disponible")]
    ConfigMissing,

    #[error("Tensor ORT: {0}")]
    Tensor(String), // ← nuevo, para Tensor::from_array
}

// ─────────────────────────────────────────────────────────────
// Conversión a respuestas Tauri
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
