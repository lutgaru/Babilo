//! Módulo LLM: integración con llama.cpp

pub mod model;
pub mod inference;

pub use model::LlmModel;
pub use inference::{InferenceEngine, InferenceState};

// Alias para compatibilidad con código existente
pub type AudioLLM = InferenceEngine;