/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Lifecycle of a practice session.
//!
//! A session encapsulates an active mode from when the user
//! presses "start" until they press "hang up". The inference
//! engine knows nothing about sessions — it only receives
//! the compound system prompt that this layer produces.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    modes::ModeConfig,
    schemas::master_system_instruction,
};

// ─── Structs that travel to the frontend ──────────────────────────────────────────

/// Capabilities of the active mode.
/// The frontend uses this to enable/disable widgets — never
/// needs to know the concrete mode type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCaps {
    pub accepts_audio: bool,
    pub accepts_text: bool,
    pub llm_initiates: bool,
}

/// Response of start_session to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session ID (UUID v4)
    pub session_id: String,
    /// Human-readable mode name (e.g. "Free Conversation")
    pub mode_id: String,
    pub mode_name: String,
    /// Capabilities so the frontend knows which widgets to show
    pub caps: SessionCaps,
    /// First line of the LLM if llm_initiates=true, None if the user speaks first
    pub opening_line: Option<String>,
}

/// Response of end_session to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub mode_name: String,
    /// Turns completed during the session
    pub turns: u32,
    /// Average score of all turns (0 if no data)
    pub average_score: u8,
}

// ─── Session logic ────────────────────────────────────────────────────────

/// Composes the final system prompt that the LLM receives.
///
/// The mode can extend or replace the master instruction.
/// For now it composes: master + "\n\n" + system_prompt of the mode.
/// In phase 3 this can become more sophisticated (templates, variables).
pub fn compose_system_prompt(mode: &dyn ModeConfig) -> String {
    let master = master_system_instruction();
    let mode_prompt = mode.system_prompt();

    // If the mode already includes the master instruction (edge case), don't duplicate
    if mode_prompt.contains("<|babilo_analysis|>") {
        return mode_prompt.to_string();
    }

    format!("{master}\n\n---\n\n{mode_prompt}")
}

/// Builds SessionInfo from an already loaded mode and a session_id.
/// The opening_line is None here — it's resolved in the command after
/// invoking the LLM if llm_initiates=true.
pub fn build_session_info(
    session_id: String,
    mode: &Arc<dyn ModeConfig>,
    opening_line: Option<String>,
) -> SessionInfo {
    SessionInfo {
        session_id,
        mode_id: mode.id().to_string(),
        mode_name: mode.name().to_string(),
        caps: SessionCaps {
            accepts_audio: mode.accepts_audio(),
            accepts_text: mode.accepts_text(),
            llm_initiates: mode.llm_initiates(),
        },
        opening_line,
    }
}

/// Builds SessionSummary when closing a session.
pub fn build_session_summary(
    session_id: String,
    mode_name: String,
    turns: u32,
    scores: &[u8],
) -> SessionSummary {
    let average_score = if scores.is_empty() {
        0
    } else {
        let sum: u32 = scores.iter().map(|&s| s as u32).sum();
        (sum / scores.len() as u32) as u8
    };

    SessionSummary {
        session_id,
        mode_name,
        turns,
        average_score,
    }
}