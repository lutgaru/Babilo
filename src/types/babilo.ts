// types/babilo.ts
export interface Correction {
  original: string;
  fixed: string;
  reason: string;
}

export interface BabiloAnalysis {
  transcription: string;
  response: string;
  corrections: Correction[];
  score: number;
  next_step_hint: string | null;
}

export type TranscriptMessage = {
  analysis: BabiloAnalysis;
  timestamp: number;
};

export interface SessionCaps {
  accepts_audio: boolean;
  accepts_text: boolean;
  llm_initiates: boolean;
}

export type SessionInfo = {
  session_id: string;
  mode_id: string;
  mode_name: string;
  caps: SessionCaps;
  opening_line: string | null;
};

export type SessionSummary = {
  session_id: string;
  mode_name: string;
  turns: number;
  average_score: number;
};

export type ModeFileInfo = {
  path: string;           // Filesystem path to the .babilo.json config
  name: string;           // Human-readable mode name
  description?: string;   // Optional description shown in UI
  caps: SessionCaps;         // Capability flags for feature toggles
};

export type BabiloEvent =
  | { type: 'sentinel_reached' }
  | { type: 'analysis'; data: BabiloAnalysis }
  | { type: 'error'; message: string }

export type AudioDevice = { name: string; id?: string }; // Adjust based on your actual backend response

export type AIState = 'idle' | 'listening' | 'thinking' | 'processing' | 'speaking';
export type Message = { role: 'user' | 'ai'; content: string; timestamp?: number };

// ── Stream Event Type (adjust to match your backend) ──
export type StreamEvent =
  | { type: 'sentinel_reached' }
  | {
    type: 'analysis';
    data: BabiloAnalysis;
  }
  | { type: 'error'; message: string };

export type AppView = 'config-list' | 'session';

// ── Settings (mirrors Rust PersistentSettings) ──

export type SeedOption = 'Random' | 'Fixed';

export interface AudioSettings {
  sample_rate: number;
  channels: number;
  chunk_duration_secs: number;
  mel_bins: number;
  window_size: number;
  hop_size: number;
}

export interface LlmSettings {
  context_size: number;
  batch_size: number;
  ubatch_size: number;
  n_gpu_layers: number;
  max_output_tokens: number;
}

export interface InferenceSettings {
  temperature: number;
  top_p: number;
  top_k: number;
  seed_option: SeedOption;
  seed_value: number;
}

export interface AeSettings {
  sample_rate: number;
  base_chunk_size: number;
}

export interface TtlSettings {
  chunk_compress_factor: number;
  latent_dim: number;
}

export interface TtsSettings {
  ae: AeSettings;
  ttl: TtlSettings;
}

export interface GuiSettings {
  theme: string;
  language: string;
}

export interface AppSettings {
  audio: AudioSettings;
  llm: LlmSettings;
  inference: InferenceSettings;
  tts: TtsSettings | null;
  gui: GuiSettings;
}