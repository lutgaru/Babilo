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