/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Módulo LLM: integración con llama.cpp

pub mod model;
pub mod inference;

pub use model::LlmModel;
pub use inference::{AudioCompactionStats, InferenceEngine, InferenceState, build_compacted_prompt};

// Alias para compatibilidad con código existente
pub type AudioLLM = InferenceEngine;