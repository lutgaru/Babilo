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

use crate::errors::SettingsError;
use serde::{Deserialize, Serialize};

use super::{AnalysisConfig, AppConfig, InferenceConfig, LlmAudioConfig, LlmConfig};

// ---------------------------------------------------------------------------
// PersistentSettings – full serializable copy of all config groups
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentSettings {
    pub audio: LlmAudioConfig,
    pub llm: LlmConfig,
    pub inference: InferenceConfig,
    #[serde(default)]
    pub analysis: AnalysisConfig,
    pub tts: Option<crate::config::TtsConfig>,
    pub gui: crate::config::GuiConfig,
}

impl PersistentSettings {
    pub fn into_app_config(self) -> AppConfig {
        AppConfig {
            audio: self.audio,
            llm: self.llm,
            analysis: self.analysis,
            tts: self.tts,
            gui: self.gui,
        }
    }

    pub fn from_app_config(cfg: AppConfig, inference: InferenceConfig) -> Self {
        PersistentSettings {
            audio: cfg.audio,
            llm: cfg.llm,
            inference,
            analysis: cfg.analysis,
            tts: cfg.tts,
            gui: cfg.gui,
        }
    }

    pub fn inference(&self) -> &InferenceConfig {
        &self.inference
    }

    pub fn inference_mut(&mut self) -> &mut InferenceConfig {
        &mut self.inference
    }
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self::from_app_config(AppConfig::default(), InferenceConfig::default())
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
