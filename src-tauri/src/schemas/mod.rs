// src-tauri/src/models/mod.rs
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, LlmError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Correction {
    pub original: String,
    pub fixed: String,
    pub reason: String,
}

pub struct BabiloStreamResult {
    pub response: String,        // available first → TTS
    pub analysis: BabiloAnalysis, // available second → UI
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BabiloAnalysis {
    pub transcription: String,
    pub corrections: Vec<Correction>,
    pub score: u8,
    pub next_step_hint: Option<String>,
}

impl BabiloAnalysis {
    pub fn from_inference(raw: &str) -> Result<Self, AppError> {
        let cleaned = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned)
            .map_err(|e| AppError::from(LlmError::Tokenization(
                format!("Invalid BabiloAnalysis JSON: {e}\nRaw: {raw}")
            )))
    }
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
        r#"You are a language learning assistant. The user will speak in their target language.

First, reply conversationally in 1-2 sentences naturally.
Then output exactly: <|babilo_analysis|>
Then output a JSON object:
{
  "transcription": "<exact transcription>",
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