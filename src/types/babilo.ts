// types/babilo.ts
export interface Correction {
  original: string;
  fixed: string;
  reason: string;
}

export interface BabiloAnalysis {
  transcription: string;
  corrections: Correction[];
  score: number;
  next_step_hint: string | null;
}

export type BabiloEvent =
  | { type: 'sentinel_reached' }
  | { type: 'analysis'; data: BabiloAnalysis }
  | { type: 'error'; message: string }