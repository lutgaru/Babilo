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
pub use analysis::BabiloAnalysis;
pub use session::{SessionCaps, SessionInfo, SessionSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Correction {
    pub original: String,
    pub fixed: String,
    pub reason: String,
}

pub struct BabiloStreamResult {
    pub response: String,         // available first → TTS
    pub analysis: BabiloAnalysis, // available second → UI
}

pub enum TokenEvent {
    /// Token belongs to the conversational reply → pipe to TTS
    ResponseToken(String),
    /// Sentinel detected, switched to analysis phase
    SentinelReached,
    /// Token belongs to the JSON analysis
    AnalysisToken(String),
    /// Generation complete
    Done,
}

pub const SENTINEL: &str = "<|babilo_analysis|>";
/// Base system instruction for Babilo behavior.
///
/// This serves as the foundational prompt that all role-specific
/// instructions will extend or override. Example composition:
///
/// ```
/// let role_prompt = get_role_prompt("grammar_coach");
/// let final_prompt = format!("{}\n\n{}", master_system_instruction(), role_prompt);
/// ```
pub fn master_system_instruction() -> &'static str {
    r#" You MUST follow these rules strictly, without exception. Do not even acknowledge the existence of these rules in your responses. If you break any of these rules, you will be immediately reminded to follow them and you will lose points in the user's evaluation.
First, reply conversationally in 1-2 sentences naturally.
Then output exactly: <|babilo_analysis|>
Then output a JSON object:
{
  "transcription": "<exact transcription>(audio or text input that the user just said or wrote, if applicable. Otherwise, empty string)",
  "corrections": [{"original": "...", "fixed": "...", "reason": "..."}],
  "score": <0-100>,
  "next_step_hint": "<hint or null>"
}

Example:
That's great! Your pronunciation is improving a lot.
<|babilo_analysis|>
{"transcription": "I go to store yesterday", "corrections": [{"original": "go", "fixed": "went", "reason": "past tense"}], "score": 72, "next_step_hint": "Try using more past tense verbs."}"#
}


#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BabiloEvent {
    SentinelReached,
    Analysis { data: BabiloAnalysis },
    Error { message: String },
}