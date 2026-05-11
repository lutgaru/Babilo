use serde::Deserialize;
use std::path::Path;
use std::fs;

// ─── JSON schema ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ModeCaps {
    #[serde(default)]
    pub llm_initiates: bool,
    #[serde(default = "default_true")]
    pub accepts_audio: bool,
    #[serde(default)]
    pub accepts_text: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct BabiloModeFile {
    pub id:             String,
    pub name:           String,
    pub description:    Option<String>,
    pub system_prompt:  String,
    pub opening_prompt: Option<String>,
    pub caps:           ModeCaps,
}

// ─── Trait ───────────────────────────────────────────────────────────────────

pub trait ModeConfig: Send + Sync {
    fn id(&self)            -> &str;
    fn name(&self)          -> &str;
    fn system_prompt(&self) -> &str;
    fn opening_prompt(&self)-> Option<&str>;
    fn llm_initiates(&self) -> bool;
    fn accepts_audio(&self) -> bool;
    fn accepts_text(&self)  -> bool;
}

pub struct BabiloMode {
    file: BabiloModeFile,
}

impl ModeConfig for BabiloMode {
    fn id(&self)             -> &str          { &self.file.id }
    fn name(&self)           -> &str          { &self.file.name }
    fn system_prompt(&self)  -> &str          { &self.file.system_prompt }
    fn opening_prompt(&self) -> Option<&str>  { self.file.opening_prompt.as_deref() }
    fn llm_initiates(&self)  -> bool          { self.file.caps.llm_initiates }
    fn accepts_audio(&self)  -> bool          { self.file.caps.accepts_audio }
    fn accepts_text(&self)   -> bool          { self.file.caps.accepts_text }
}

// ─── Loader ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ModeLoadError {
    #[error("No se pudo leer el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON inválido en '{path}': {source}")]
    Parse { path: String, source: serde_json::Error },
}

/// Loads a mode from an absolute or relative path to a .babilo JSON file.
///
/// Example:
/// ```rust
/// let mode = load_mode("modes/conversation.babilo.json")?;
/// ```
pub fn load_mode(path: impl AsRef<Path>) -> Result<BabiloMode, ModeLoadError> {
    let path = path.as_ref();
    let raw  = fs::read_to_string(path)?;
    let file = serde_json::from_str::<BabiloModeFile>(&raw)
        .map_err(|e| ModeLoadError::Parse {
            path:   path.display().to_string(),
            source: e,
        })?;

    Ok(BabiloMode { file })
}