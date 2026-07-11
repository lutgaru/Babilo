/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Configuración centralizada de la aplicación

pub mod settings_loader;

use serde::{Deserialize, Serialize};

/// Configuración de captura y procesamiento de audio
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmAudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_secs: u32,
    pub mel_bins: usize,
    pub window_size: usize,
    pub hop_size: usize,
}

impl Default for LlmAudioConfig {
    fn default() -> Self {
        Self::gemma4()
    }
}

impl LlmAudioConfig {
    /// Configuración optimizada para Gemma 4: 30s chunks, 16kHz, mono
    pub fn gemma4() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            chunk_duration_secs: 30,
            mel_bins: 128,
            window_size: 320, // 20ms @ 16kHz
            hop_size: 160,    // 10ms hop
        }
    }

    pub fn samples_per_chunk(&self) -> usize {
        self.chunk_duration_secs as usize * self.sample_rate as usize
    }
}

/// Configuración del modelo LLM
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub context_size: u32,
    pub batch_size: u32,
    pub ubatch_size: u32,
    pub n_gpu_layers: u32,
    pub max_output_tokens: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            context_size: 4096,
            batch_size: 2048,
            ubatch_size: 512,
            n_gpu_layers: 99,
            max_output_tokens: 10000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub context_size: u32,
    pub max_output_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub seed_option: SeedOption,
    pub seed_value: u32,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            context_size: 2000,
            max_output_tokens: 512,
            temperature: 0.3,
            top_p: 0.8,
            top_k: 10,
            seed_option: SeedOption::Random,
            seed_value: 7,
        }
    }
}

/// Configuración del motor TTS
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsConfig {
    pub ae: AeConfig,
    pub ttl: TtlConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SeedOption {
    Random,
    Fixed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub seed_option: SeedOption,
    pub seed_value: u32,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            seed_option: SeedOption::Random,
            seed_value: 7,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AeConfig {
    pub sample_rate: i32,
    pub base_chunk_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuiConfig {
    pub language: String,
    pub theme: String,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "light".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtlConfig {
    pub chunk_compress_factor: i32,
    pub latent_dim: i32,
}

impl TtsConfig {
    pub fn latent_dim_compressed(&self) -> usize {
        (self.ttl.latent_dim * self.ttl.chunk_compress_factor) as usize
    }

    pub fn chunk_size_compressed(&self) -> usize {
        (self.ae.base_chunk_size * self.ttl.chunk_compress_factor) as usize
    }
}

/// Configuración global de la aplicación
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: LlmAudioConfig,
    pub llm: LlmConfig,
    pub analysis: AnalysisConfig,
    pub tts: Option<TtsConfig>,
    pub gui: GuiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: LlmAudioConfig::default(),
            llm: LlmConfig::default(),
            analysis: AnalysisConfig::default(),
            tts: None,
            gui: GuiConfig::default(),
        }
    }
}

