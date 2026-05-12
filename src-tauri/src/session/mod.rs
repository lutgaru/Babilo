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
use uuid::Uuid;

use crate::{
    errors::{AppResult, SessionError},
    modes::{load_mode, ModeConfig},
    schemas::master_system_instruction,
};

// ─── Session States ─────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Initialized,
    Active,
    Paused,
    Ended,
}

// ─── Structs sent to frontend ──────────────────────────

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub mode_name: String,
    pub turns: u32,
    pub average_score: u8,
}

// ─── Session Manager ─────────────────────────────────────────

pub struct SessionManager {
    active_session: Option<Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: None,
        }
    }

    /// Start a new session with the specified mode
    pub fn start_session(&mut self, path: &str) -> AppResult<SessionInfo> {
        // Check if there is already an active session
        if self.active_session.is_some() {
            return Err(SessionError::AlreadyActive.into());
        }

        // Load the mode
        let mode = Arc::new(load_mode(path).map_err(|e| SessionError::LoadError(e.to_string()))?);

        // Verify the mode is available for sessions
        // (you could add additional validation logic here)

        // Generate UUID for the session
        let session_id = Uuid::new_v4().to_string();

        // Compose the system prompt
        let system_prompt =
            compose_system_prompt(mode.as_ref()).map_err(|e| SessionError::PromptComposition(e))?;

        // Create the session
        let session = Session {
            id: session_id.clone(),
            mode,
            state: SessionState::Initialized,
            system_prompt,
            turns: 0,
            scores: Vec::new(),
        };

        // Generate opening_line if the mode requires it
        let opening_line = if session.mode.llm_initiates() {
            // The LLM call to generate the first line would go here
            // For now, None - resolved in the command layer
            None
        } else {
            None
        };

        let session_info =
            build_session_info(session_id.clone(), &session.mode, opening_line.clone());

        // Activate the session
        self.active_session = Some(session);

        Ok(session_info)
    }

    /// Get information about the active session
    pub fn get_active_session(&self) -> AppResult<&Session> {
        self.active_session
            .as_ref()
            .ok_or_else(|| SessionError::NotInitialized.into())
    }

    /// Get mutable information about the active session
    pub fn get_active_session_mut(&mut self) -> AppResult<&mut Session> {
        self.active_session
            .as_mut()
            .ok_or_else(|| SessionError::NotInitialized.into())
    }

    /// End the active session and return a summary
    pub fn end_session(&mut self) -> AppResult<SessionSummary> {
        let session = self
            .active_session
            .take()
            .ok_or_else(|| SessionError::NotFound("no active session".into()))?;

        // Validate state transition
        if session.state == SessionState::Ended {
            return Err(SessionError::InvalidStateTransition {
                from: "Ended".into(),
                to: "Ended".into(),
            }
            .into());
        }

        Ok(build_session_summary(
            session.id,
            session.mode.name().to_string(),
            session.turns,
            &session.scores,
        ))
    }

    /// Pause the active session
    pub fn pause_session(&mut self) -> AppResult<()> {
        let session = self.get_active_session_mut()?;

        if session.state != SessionState::Active {
            return Err(SessionError::InvalidStateTransition {
                from: format!("{:?}", session.state),
                to: "Paused".into(),
            }
            .into());
        }

        session.state = SessionState::Paused;
        Ok(())
    }

    /// Resume the paused session
    pub fn resume_session(&mut self) -> AppResult<()> {
        let session = self.get_active_session_mut()?;

        if session.state != SessionState::Paused {
            return Err(SessionError::InvalidStateTransition {
                from: format!("{:?}", session.state),
                to: "Active".into(),
            }
            .into());
        }

        session.state = SessionState::Active;
        Ok(())
    }

    /// Record a completed turn with its score
    pub fn record_turn(&mut self, score: u8) -> AppResult<()> {
        let session = self.get_active_session_mut()?;

        if session.state != SessionState::Active {
            return Err(SessionError::OperationNotAllowed(
                "cannot record turn in non-active session".into(),
            )
            .into());
        }

        session.turns += 1;
        session.scores.push(score);
        Ok(())
    }

    pub fn require_mode(&self) -> AppResult<Arc<dyn ModeConfig>> {
        self.active_session
            .as_ref()
            .map(|s| Arc::clone(&s.mode))
            .ok_or_else(|| SessionError::NotInitialized.into())
    }
}

// ─── Internal Session struct ─────────────────────────────────

pub struct Session {
    pub id: String,
    pub mode: Arc<dyn ModeConfig>,
    pub state: SessionState,
    pub system_prompt: String,
    pub turns: u32,
    pub scores: Vec<u8>,
}

// ─── Helper functions ────────────────────────────────────────

pub fn compose_system_prompt(mode: &dyn ModeConfig) -> Result<String, String> {
    let master = master_system_instruction();
    let mode_prompt = mode.system_prompt();

    if mode_prompt.contains("<|babilo_analysis|>") {
        return Ok(mode_prompt.to_string());
    }

    Ok(format!("{master}\n\n---\n\n{mode_prompt}"))
}

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
