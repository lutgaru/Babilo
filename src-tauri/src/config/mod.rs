//! Configuración centralizada de la aplicación

use serde::{Deserialize, Serialize};

/// Configuración de captura y procesamiento de audio
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_secs: u32,
    pub mel_bins: usize,
    pub window_size: usize,
    pub hop_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self::gemma4()
    }
}

impl AudioConfig {
    /// Configuración optimizada para Gemma 4: 30s chunks, 16kHz, mono
    pub fn gemma4() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            chunk_duration_secs: 30,
            mel_bins: 128,
            window_size: 320,  // 20ms @ 16kHz
            hop_size: 160,     // 10ms hop
        }
    }

    pub fn samples_per_chunk(&self) -> usize {
        self.chunk_duration_secs as usize * self.sample_rate as usize
    }
}

/// Configuración del modelo LLM
#[derive(Clone, Debug)]
pub struct LlmConfig {
    pub context_size: u32,
    pub batch_size: u32,
    pub ubatch_size: u32,
    pub n_gpu_layers: u32,
    pub max_output_tokens: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            context_size: 4096,
            batch_size: 2048,
            ubatch_size: 512,
            n_gpu_layers: 99, // Offload máximo a GPU
            max_output_tokens: 150,
        }
    }
}

/// Configuración del motor TTS
#[derive(Clone, Debug, Deserialize)]
pub struct TtsConfig {
    pub ae: AeConfig,
    pub ttl: TtlConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AeConfig {
    pub sample_rate: i32,
    pub base_chunk_size: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TtlConfig {
    pub chunk_compress_factor: i32,
    pub latent_dim: i32,
}

impl TtsConfig {
    pub fn latent_dim_compressed(&self) -> usize {
        (self.ttl.latent_dim * self.ttl.chunk_compress_factor) as usize
    }

    pub fn chunk_size_compressed(&self) -> usize {
        (self.ae.base_chunk_size * self.ttl.chunk_compress_factor) as usize
    }
}

/// Configuración global de la aplicación
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub llm: LlmConfig,
    pub tts: Option<TtsConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            llm: LlmConfig::default(),
            tts: None, // Se carga desde tts.json en runtime
        }
    }
}