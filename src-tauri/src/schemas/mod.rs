/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

pub mod analysis;
pub mod session;
pub mod settings;
pub use analysis::{BabiloAnalysis, Correction};
pub use session::{SessionCaps, SessionInfo, SessionSummary};
pub use settings::PersistentSettings;
use serde::{Deserialize, Serialize};

pub enum TokenEvent {
    Token(String),
    Done,
}

/// System instruction for the conversation (response) phase.
/// The model should reply conversationally without any analysis.
pub fn conversation_system_instruction() -> &'static str {
    r#" You MUST follow these rules strictly, without exception.
Reply conversationally in 1-2 sentences naturally.
Do not output any JSON or analysis — just respond as a conversation partner."#
}

/// System instruction for the analysis phase.
/// The model receives the user input and the AI response, and must
/// produce ONLY a JSON analysis object.
pub fn analysis_system_instruction() -> &'static str {
    r#" You are an English language analyst. Based ONLY on the user input and the AI response provided below, output a JSON object with this exact structure:

{
  "transcription": "<exact transcription of the user input>",
  "corrections": [{"original": "<incorrect part>", "fixed": "<corrected version>", "reason": "<brief explanation>"}],
  "score": <0-100>,
  "next_step_hint": "<suggestion for improvement or null>"
}

Rules:
- transcription: verbatim copy of the user's input
- corrections: list any grammar/vocabulary/pronunciation errors (can be empty array)
- score: 0-100 rating of the user's language quality
- next_step_hint: a short tip to help the user improve, or null if not needed
- Output ONLY valid JSON, no explanation before or after"#
}


#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BabiloEvent {
    Response { text: String },
    Analysis { data: BabiloAnalysis },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiState {
    Idle,
    Listening,
    Thinking,
    Speaking,
}
