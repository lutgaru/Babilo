/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Correction {
    pub original: String,
    pub fixed: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabiloAnalysis {
    pub transcription: String,
    pub corrections: Vec<Correction>,
    pub score: u8,
    pub next_step_hint: Option<String>,
}

use crate::errors::{AppError, LlmError};

pub struct BabiloAnalysisBuilder {
    transcription: Option<String>,
    corrections: Option<Vec<Correction>>,
    score: Option<u8>,
    next_step_hint: Option<String>,
}

impl Default for BabiloAnalysisBuilder {
    fn default() -> Self {
        Self {
            transcription: None,
            corrections: None,
            score: None,
            next_step_hint: None,
        }
    }
}

impl BabiloAnalysisBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transcription(mut self, transcription: String) -> Self {
        self.transcription = Some(transcription);
        self
    }

    pub fn with_corrections(mut self, corrections: Vec<Correction>) -> Self {
        self.corrections = Some(corrections);
        self
    }

    pub fn with_score(mut self, score: u8) -> Self {
        self.score = Some(score);
        self
    }

    pub fn with_next_step_hint(mut self, hint: String) -> Self {
        self.next_step_hint = Some(hint);
        self
    }

    pub fn with_json_payload(mut self, raw: &str) -> Result<Self, AppError> {
        #[derive(Deserialize)]
        struct PartialAnalysis {
            transcription: Option<String>,
            corrections: Option<Vec<Correction>>,
            score: Option<u8>,
            next_step_hint: Option<String>,
        }

        let cleaned = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let partial: PartialAnalysis = serde_json::from_str(cleaned).map_err(|e| {
            AppError::from(LlmError::Tokenization(format!(
                "Invalid BabiloAnalysis JSON: {e}\nRaw: {raw}"
            )))
        })?;

        if self.transcription.is_none() {
            self.transcription = partial.transcription;
        }
        if self.corrections.is_none() {
            self.corrections = partial.corrections;
        }
        if self.score.is_none() {
            self.score = partial.score;
        }
        if self.next_step_hint.is_none() {
            self.next_step_hint = partial.next_step_hint;
        }

        Ok(self)
    }

    pub fn build(self) -> Result<BabiloAnalysis, AppError> {
        Ok(BabiloAnalysis {
            transcription: self
                .transcription
                .ok_or_else(|| AppError::from(LlmError::MissingField("transcription".into())))?,
            corrections: self
                .corrections
                .ok_or_else(|| AppError::from(LlmError::MissingField("corrections".into())))?,
            score: self
                .score
                .ok_or_else(|| AppError::from(LlmError::MissingField("score".into())))?,
            next_step_hint: self.next_step_hint,
        })
    }
}

impl BabiloAnalysis {
    pub fn builder() -> BabiloAnalysisBuilder {
        BabiloAnalysisBuilder::new()
    }
}
