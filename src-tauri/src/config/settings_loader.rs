/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Loader/saver for configuration to/from a JSON file

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::GuiConfig;

use super::{AppConfig, AudioConfig, InferenceConfig, LlmConfig, SeedOption};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors when loading/saving configuration
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error serializing/deserializing configuration: {0}")]
    Json(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// PersistentSettings – full serde copy of all config groups
// ---------------------------------------------------------------------------

/// Serializable representation of all application configuration.
/// Used only for JSON interchange; runtime config remains [`AppConfig`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentSettings {
    pub audio: PersistentAudioConfig,
    pub llm: PersistentLlmConfig,
    pub inference: PersistentInferenceConfig,
    pub tts: Option<PersistentTtsConfig>,
    pub gui: PersistentGuiConfig,
}

impl PersistentSettings {
    /// Converts to [`AppConfig`] (inference is dropped here; access it
    /// via [`Self::inference`] / [`Self::inference_mut`]).
    pub fn into_app_config(self) -> AppConfig {
        AppConfig {
            audio: self.audio.into(),
            llm: self.llm.into(),
            tts: self.tts.map(PersistentTtsConfig::into_tts),
            gui: self.gui.into(),
        }
    }

    /// Builds from an [`AppConfig`] plus a separate [`InferenceConfig`]
    /// (since `AppConfig` does not carry inference).
    pub fn from_app_config(cfg: AppConfig, inference: InferenceConfig) -> Self {
        PersistentSettings {
            audio: cfg.audio.into(),
            llm: cfg.llm.into(),
            inference: inference.into(),
            tts: cfg.tts.map(PersistentTtsConfig::from_tts),
            gui: PersistentGuiConfig::from(cfg.gui),
        }
    }

    pub fn inference(&self) -> &PersistentInferenceConfig {
        &self.inference
    }

    pub fn inference_mut(&mut self) -> &mut PersistentInferenceConfig {
        &mut self.inference
    }
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self::from_app_config(AppConfig::default(), InferenceConfig::default())
    }
}

// -- GUI --------------------------------------------------------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentGuiConfig {
    pub theme: String,
    pub language: String,
}

impl From<PersistentGuiConfig> for GuiConfig {
    fn from(p: PersistentGuiConfig) -> Self {
        GuiConfig {
            theme: p.theme,
            language: p.language,
        }
    }
}

impl From<GuiConfig> for PersistentGuiConfig {
    fn from(c: GuiConfig) -> Self {
        PersistentGuiConfig {
            theme: c.theme,
            language: c.language,
        }
    }
}

// -- Audio ------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentAudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_secs: u32,
    pub mel_bins: usize,
    pub window_size: usize,
    pub hop_size: usize,
}

impl From<PersistentAudioConfig> for AudioConfig {
    fn from(p: PersistentAudioConfig) -> Self {
        AudioConfig {
            sample_rate: p.sample_rate,
            channels: p.channels,
            chunk_duration_secs: p.chunk_duration_secs,
            mel_bins: p.mel_bins,
            window_size: p.window_size,
            hop_size: p.hop_size,
        }
    }
}

impl From<AudioConfig> for PersistentAudioConfig {
    fn from(c: AudioConfig) -> Self {
        PersistentAudioConfig {
            sample_rate: c.sample_rate,
            channels: c.channels,
            chunk_duration_secs: c.chunk_duration_secs,
            mel_bins: c.mel_bins,
            window_size: c.window_size,
            hop_size: c.hop_size,
        }
    }
}

// -- LLM --------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentLlmConfig {
    pub context_size: u32,
    pub batch_size: u32,
    pub ubatch_size: u32,
    pub n_gpu_layers: u32,
    pub max_output_tokens: usize,
}

impl From<PersistentLlmConfig> for LlmConfig {
    fn from(p: PersistentLlmConfig) -> Self {
        LlmConfig {
            context_size: p.context_size,
            batch_size: p.batch_size,
            ubatch_size: p.ubatch_size,
            n_gpu_layers: p.n_gpu_layers,
            max_output_tokens: p.max_output_tokens,
        }
    }
}

impl From<LlmConfig> for PersistentLlmConfig {
    fn from(c: LlmConfig) -> Self {
        PersistentLlmConfig {
            context_size: c.context_size,
            batch_size: c.batch_size,
            ubatch_size: c.ubatch_size,
            n_gpu_layers: c.n_gpu_layers,
            max_output_tokens: c.max_output_tokens,
        }
    }
}

// -- Inference --------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentInferenceConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub seed_option: SeedOption,
    pub seed_value: u32,
}

impl From<PersistentInferenceConfig> for InferenceConfig {
    fn from(p: PersistentInferenceConfig) -> Self {
        InferenceConfig {
            temperature: p.temperature,
            top_p: p.top_p,
            top_k: p.top_k,
            seed_option: p.seed_option,
            seed_value: p.seed_value,
        }
    }
}

impl From<InferenceConfig> for PersistentInferenceConfig {
    fn from(c: InferenceConfig) -> Self {
        PersistentInferenceConfig {
            temperature: c.temperature,
            top_p: c.top_p,
            top_k: c.top_k,
            seed_option: c.seed_option,
            seed_value: c.seed_value,
        }
    }
}

// -- TTS --------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentAeConfig {
    pub sample_rate: i32,
    pub base_chunk_size: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentTtlConfig {
    pub chunk_compress_factor: i32,
    pub latent_dim: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentTtsConfig {
    pub ae: PersistentAeConfig,
    pub ttl: PersistentTtlConfig,
}

impl PersistentTtsConfig {
    fn from_tts(tts: super::TtsConfig) -> Self {
        PersistentTtsConfig {
            ae: PersistentAeConfig {
                sample_rate: tts.ae.sample_rate,
                base_chunk_size: tts.ae.base_chunk_size,
            },
            ttl: PersistentTtlConfig {
                chunk_compress_factor: tts.ttl.chunk_compress_factor,
                latent_dim: tts.ttl.latent_dim,
            },
        }
    }

    fn into_tts(self) -> super::TtsConfig {
        super::TtsConfig {
            ae: super::AeConfig {
                sample_rate: self.ae.sample_rate,
                base_chunk_size: self.ae.base_chunk_size,
            },
            ttl: super::TtlConfig {
                chunk_compress_factor: self.ttl.chunk_compress_factor,
                latent_dim: self.ttl.latent_dim,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// SettingsLoader
// ---------------------------------------------------------------------------

/// Persistent JSON configuration loader/saver.
///
/// # Example
///
/// ```ignore
/// let loader = SettingsLoader::new();
/// let settings = loader.load().unwrap();
/// settings.audio.sample_rate = 44100;
/// loader.save(&settings).unwrap();
/// ```
pub struct SettingsLoader {
    path: std::path::PathBuf,
}

fn config_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Project root not found")
        .join("config")
}

impl SettingsLoader {
    /// Creates a new loader pointing at `path`.

    pub fn new() -> Self {
        Self {
            path: config_dir().join("settings.json"),
        }
    }

    /// Loads configuration from the JSON file.
    ///
    /// Returns default values if the file does not exist.
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load(&self) -> Result<PersistentSettings, SettingsError> {
        if !self.path.exists() {
            return Ok(PersistentSettings::default());
        }
        let data = std::fs::read_to_string(&self.path)?;
        let settings: PersistentSettings = serde_json::from_str(&data)?;
        Ok(settings)
    }

    /// Saves configuration to the JSON file (pretty-printed).
    ///
    /// Creates the parent directory if it does not exist.
    pub fn save(&self, settings: &PersistentSettings) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }
}
