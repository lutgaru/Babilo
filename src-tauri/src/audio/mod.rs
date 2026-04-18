pub mod capture;
pub mod mel_preprocessor;

// Re-export para facilitar imports
pub use capture::{AudioCapture, AudioConfig};
pub use mel_preprocessor::MelPreprocessor;