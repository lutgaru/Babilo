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
use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::{AppResult, ModeError};

// ─── JSON schema ─────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeFileInfo {
    pub path: String,
    pub name: String,
    pub caps: ModeCaps,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ModeCaps {
    #[serde(default)]
    pub llm_initiates: bool,
    #[serde(default = "default_true")]
    pub accepts_audio: bool,
    #[serde(default)]
    pub accepts_text: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct BabiloModeFile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub mode_prompt: String,
    pub role_prompt: Option<String>,
    pub opening_prompt: Option<String>,
    pub caps: ModeCaps,
}

// ─── Trait ───────────────────────────────────────────────────
pub trait ModeConfig: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn mode_prompt(&self) -> &str;
    fn role_prompt(&self) -> Option<&str>;
    fn opening_prompt(&self) -> Option<&str>;
    fn llm_initiates(&self) -> bool;
    fn accepts_audio(&self) -> bool;
    fn accepts_text(&self) -> bool;
}

pub struct BabiloMode {
    file: BabiloModeFile,
}

impl ModeConfig for BabiloMode {
    fn id(&self) -> &str {
        &self.file.id
    }
    fn name(&self) -> &str {
        &self.file.name
    }
    fn mode_prompt(&self) -> &str {
        &self.file.mode_prompt
    }
    fn role_prompt(&self) -> Option<&str> {
        self.file.role_prompt.as_deref()
    }
    fn opening_prompt(&self) -> Option<&str> {
        self.file.opening_prompt.as_deref()
    }
    fn llm_initiates(&self) -> bool {
        self.file.caps.llm_initiates
    }
    fn accepts_audio(&self) -> bool {
        self.file.caps.accepts_audio
    }
    fn accepts_text(&self) -> bool {
        self.file.caps.accepts_text
    }
}

// ─── Loader con errores unificados ───────────────────────────

pub fn modes_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Project root not found")
        .join("modes")
}

/// Loads a mode from a path to a .babilo JSON file.
pub fn load_mode(path: impl AsRef<Path>) -> AppResult<BabiloMode> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    let raw = fs::read_to_string(path).map_err(|e| ModeError::IoRead {
        path: path_str.clone(),
        source: e,
    })?;

    let file: BabiloModeFile = serde_json::from_str(&raw).map_err(|e| ModeError::Parse {
        path: path_str.clone(),
        source: e,
    })?;

    // Validación básica de campos requeridos
    if file.id.is_empty() {
        return Err(ModeError::MissingField {
            path: path_str.clone(),
            field: "id".into(),
        }
        .into());
    }
    if file.name.is_empty() {
        return Err(ModeError::MissingField {
            path: path_str.clone(),
            field: "name".into(),
        }
        .into());
    }

    Ok(BabiloMode { file })
}

pub fn list_modes() -> AppResult<Vec<ModeFileInfo>> {
    let dir = modes_dir();

    if !dir.exists() {
        return Err(ModeError::DirectoryNotFound(dir.display().to_string()).into());
    }

    let mut modes = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|e| ModeError::IoRead {
        path: dir.display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| ModeError::IoRead {
            path: dir.display().to_string(),
            source: e,
        })?;

        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match load_mode(&path) {
                Ok(mode) => modes.push(ModeFileInfo {
                    path: path.display().to_string(),
                    name: mode.name().to_string(),
                    caps: ModeCaps {
                        llm_initiates: mode.llm_initiates(),
                        accepts_audio: mode.accepts_audio(),
                        accepts_text: mode.accepts_text(),
                    },
                }),
                Err(e) => {
                    eprintln!("Error loading mode from '{}': {e}", path.display());
                    continue;
                }
            }
        }
    }

    Ok(modes)
}

/// Load a specific mode by ID from the modes directory
pub fn load_mode_by_id(mode_id: &str) -> AppResult<BabiloMode> {
    for entry in fs::read_dir(modes_dir()).map_err(|e| ModeError::IoRead {
        path: modes_dir().display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| ModeError::IoRead {
            path: modes_dir().display().to_string(),
            source: e,
        })?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(mode) = load_mode(&path) {
                if mode.id() == mode_id {
                    return Ok(mode);
                }
            }
        }
    }

    Err(ModeError::NotFound(mode_id.to_string()).into())
}
