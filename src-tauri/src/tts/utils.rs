/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Utilidades para el módulo TTS

use crate::errors::{AppError, TtsError};
use serde::Deserialize;
use std::path::PathBuf;

/// Procesador de texto a índices Unicode
pub struct UnicodeProcessor {
    indexer: Vec<i64>,
}

impl UnicodeProcessor {
    pub fn new(assets_dir: &PathBuf) -> Result<Self, AppError> {
        let path = assets_dir.join("onnx").join("unicode_indexer.json");
        let json = std::fs::read_to_string(&path).map_err(|_| TtsError::ConfigMissing)?;
        let indexer: Vec<i64> =
            serde_json::from_str(&json).map_err(|e| TtsError::SessionLoad(e.to_string()))?;
        Ok(Self { indexer })
    }

    pub fn encode(&self, text: &str, lang: &str) -> Vec<i64> {
        let wrapped = format!("<{}>{}</{}>", lang, text, lang);
        wrapped
            .chars()
            .map(|c| {
                let idx = c as usize;
                if idx < self.indexer.len() {
                    self.indexer[idx]
                } else {
                    -1
                }
            })
            .collect()
    }
}

/// Estilo de voz para TTS
#[derive(Deserialize, Debug)]
pub struct VoiceStyle {
    pub style_ttl: StyleTensor,
    pub style_dp: StyleTensor,
}

#[derive(Deserialize, Debug)]
pub struct StyleTensor {
    pub data: Vec<Vec<Vec<f32>>>,
    pub dims: Vec<usize>,
}

impl StyleTensor {
    pub fn flatten(&self) -> (Vec<f32>, Vec<usize>) {
        let flat = self
            .data
            .iter()
            .flat_map(|d2| d2.iter().flat_map(|d3| d3.iter().copied()))
            .collect();
        (flat, self.dims.clone())
    }
}

/// Cargar estilo de voz desde archivo JSON
pub fn load_voice_style(assets_dir: &PathBuf, voice_id: &str) -> Result<VoiceStyle, AppError> {
    let path = assets_dir
        .join("voice_styles")
        .join(format!("{}.json", voice_id));

    let json = std::fs::read_to_string(&path)
        .map_err(|_| TtsError::VoiceLoad(format!("Voice '{}' not found", voice_id)))?;

    serde_json::from_str(&json).map_err(|e| TtsError::VoiceLoad(e.to_string()).into())
}
