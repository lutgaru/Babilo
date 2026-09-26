/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Lógica de inferencia: tokenización, generación, manejo de contexto

use std::time::SystemTime;

use crate::{
    config::{AnalysisConfig, InferenceConfig, LlmConfig, SeedOption},
    errors::{AppError, LlmError},
    llama::model::LlmModel,
    schemas::TokenEvent,
};
use llama_cpp_2::{
    context::LlamaContext,
    llama_batch::LlamaBatch,
    model::AddBos,
    mtmd::{MtmdBitmap, MtmdInputText},
    sampling::LlamaSampler,
    token::LlamaToken,
};

#[derive(Default)]
pub struct InferenceState {
    pub n_past: i32,
    pub system_prompt_evaluated: bool,
}

/// Stats returned by `compact_audio_turn`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioCompactionStats {
    /// KV positions before compaction (== n_past after response generation).
    pub n_past_before: i32,
    /// KV positions after compaction.
    pub n_past_after: i32,
    /// Tokens freed (before - after, can be 0 if skipped).
    pub tokens_freed: i32,
    /// Whether compaction actually ran (false when skipped).
    pub compacted: bool,
}

pub struct InferenceEngine {
    model: LlmModel,
    state: InferenceState,
    last_context_cleared: bool,
    /// Start position (n_past) of the current turn's audio prompt in the
    /// main context. Set by `infer_audio_streaming`, consumed by
    /// `compact_audio_turn`. `None` for text turns or after compaction/reset.
    pending_audio_start: Option<i32>,
}

impl InferenceEngine {
    pub fn new(model: LlmModel) -> Self {
        Self {
            model,
            state: InferenceState::default(),
            last_context_cleared: false,
            pending_audio_start: None,
        }
    }

    // ── Response inference (uses main 200k context) ──────────

    pub fn infer_audio_streaming(
        &mut self,
        audio_pcm: &[f32],
        prompt: &str,
        on_token: impl FnMut(TokenEvent),
    ) -> Result<(), AppError> {
        let add_special = !self.state.system_prompt_evaluated;
        let audio_bitmap = MtmdBitmap::from_audio_data(audio_pcm)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        let chunks = {
            let mtmd = self
                .model
                .mtmd_context()
                .ok_or(LlmError::MtmdInit("No mmproj loaded".into()))?;

            mtmd.tokenize(
                MtmdInputText {
                    text: prompt.to_string(),
                    add_special,
                    parse_special: true,
                },
                &[&audio_bitmap],
            )
            .map_err(|e| LlmError::Tokenization(e.to_string()))?
        };

        let total_tokens = chunks.total_tokens();
        let n_ctx = self.model.n_ctx();
        let config = self.model.config().clone();
        let inference_config = self.model.inference_config().clone();

        let (ctx, mtmd) = self.model.split_ctx_mtmd()?;

        let cleared = ensure_context_space(ctx, &mut self.state, n_ctx, total_tokens)?;
        self.last_context_cleared = cleared;

        // Track where this audio turn starts in the KV cache so it can be
        // replaced by its transcription later (see `compact_audio_turn`).
        // If the context was wiped above, n_past is 0 — still correct.
        let audio_start = self.state.n_past;

        let new_n_past = chunks
            .eval_chunks(mtmd, ctx, self.state.n_past, 0, 512, true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        self.state.n_past = new_n_past;
        self.state.system_prompt_evaluated = true;
        self.pending_audio_start = Some(audio_start);

        generate_streaming(ctx, &mut self.state, &config, &inference_config, on_token)?;

        Ok(())
    }

    pub fn infer_text_streaming(
        &mut self,
        prompt: &str,
        on_token: impl FnMut(TokenEvent),
    ) -> Result<(), AppError> {
        let add_bos = bos_flag(&self.state);

        let tokens = self
            .model
            .model()
            .str_to_token(prompt, add_bos)
            .map_err(|e| LlmError::Tokenization(e.to_string()))?;

        let n_ctx = self.model.n_ctx();
        let config = self.model.config().clone();
        let inference_config = self.model.inference_config().clone();

        let ctx = self.model.ctx_mut()?;

        let cleared = ensure_context_space(ctx, &mut self.state, n_ctx, tokens.len())?;
        self.last_context_cleared = cleared;
        decode_tokens(ctx, &mut self.state, &tokens)?;

        self.state.system_prompt_evaluated = true;
        // Text turns need no compaction.
        self.pending_audio_start = None;

        generate_streaming(ctx, &mut self.state, &config, &inference_config, on_token)?;

        Ok(())
    }

    // ── Analysis inference (uses small 2k context, reset each turn) ──

    pub fn infer_audio_analysis_streaming(
        &mut self,
        audio_pcm: &[f32],
        analysis_prompt: &str,
        on_token: impl FnMut(TokenEvent),
    ) -> Result<(), AppError> {
        self.model.reset_analysis_context()?;

        let analysis_config = self.model.analysis_config().clone();
        let config = self.model.config().clone();
        let n_ctx = self.model.analysis_n_ctx();
        let n_ubatch = self.model.analysis_n_ubatch();

        let audio_bitmap = MtmdBitmap::from_audio_data(audio_pcm)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        let chunks = {
            let mtmd = self
                .model
                .mtmd_context()
                .ok_or(LlmError::MtmdInit("No mmproj loaded".into()))?;

            mtmd.tokenize(
                MtmdInputText {
                    text: analysis_prompt.to_string(),
                    add_special: true,
                    parse_special: true,
                },
                &[&audio_bitmap],
            )
            .map_err(|e| LlmError::Tokenization(e.to_string()))?
        };

        let total_tokens = chunks.total_tokens();

        if total_tokens > n_ctx as usize {
            return Err(LlmError::ContextFull.into());
        }

        let (ctx, mtmd) = self.model.split_analysis_ctx_mtmd()?;
        let mut analysis_state = InferenceState::default();

        let new_n_past = chunks
            .eval_chunks(mtmd, ctx, analysis_state.n_past, 0, n_ubatch as i32, true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        analysis_state.n_past = new_n_past;

        generate_streaming(
            ctx,
            &mut analysis_state,
            &config,
            &analysis_config,
            on_token,
        )?;

        Ok(())
    }

    pub fn infer_analysis_streaming(
        &mut self,
        analysis_prompt: &str,
        on_token: impl FnMut(TokenEvent),
    ) -> Result<(), AppError> {
        self.model.reset_analysis_context()?;

        let analysis_config = self.model.analysis_config().clone();
        let config = self.model.config().clone();
        let n_ctx = self.model.analysis_n_ctx();

        let tokens = self
            .model
            .model()
            .str_to_token(analysis_prompt, AddBos::Always)
            .map_err(|e| LlmError::Tokenization(e.to_string()))?;

        if tokens.len() > n_ctx as usize {
            return Err(LlmError::ContextFull.into());
        }

        let ctx = self.model.analysis_ctx_mut()?;
        let mut analysis_state = InferenceState::default();
        decode_tokens(ctx, &mut analysis_state, &tokens)?;

        generate_streaming(
            ctx,
            &mut analysis_state,
            &config,
            &analysis_config,
            on_token,
        )?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), AppError> {
        self.model.reset_context()?;
        self.model.reset_analysis_context()?;
        self.state = InferenceState::default();
        self.last_context_cleared = false;
        self.pending_audio_start = None;
        Ok(())
    }

    /// Replace the last audio turn's tokens in the main KV cache with its
    /// transcription (text).
    ///
    /// Strategy (truncate + re-decode):
    /// 1. `original_prompt` is the exact prompt string evaluated in phase 1
    ///    (contains the MTMD audio marker). Replacing the marker with the
    ///    transcription preserves the system prompt + turn framing.
    /// 2. Truncate the KV cache to `pending_audio_start` (drops audio
    ///    embeddings + the just-generated response).
    /// 3. Re-decode `compacted_prompt + response_text` as plain text.
    ///
    /// Result: history keeps full semantic content (transcription + reply)
    /// at ~20-60 tokens instead of ~1000+ audio tokens.
    ///
    /// No-op (returns `compacted: false`) when there is no pending audio
    /// turn or the transcription is empty — the audio KV is kept so no
    /// information is lost.
    pub fn compact_audio_turn(
        &mut self,
        original_prompt: &str,
        transcription: &str,
        response_text: &str,
    ) -> Result<AudioCompactionStats, AppError> {
        let n_past_before = self.state.n_past;
        let audio_start = match self.pending_audio_start {
            Some(pos) => pos,
            None => {
                return Ok(AudioCompactionStats {
                    n_past_before,
                    n_past_after: n_past_before,
                    tokens_freed: 0,
                    compacted: false,
                });
            }
        };

        let transcription = transcription.trim();
        if transcription.is_empty() {
            // Cannot replace audio with nothing — keep audio KV.
            self.pending_audio_start = None;
            return Ok(AudioCompactionStats {
                n_past_before,
                n_past_after: n_past_before,
                tokens_freed: 0,
                compacted: false,
            });
        }

        let compacted_prompt = build_compacted_prompt(original_prompt, transcription);
        let mut full_replacement = compacted_prompt;
        let response_trimmed = response_text.trim();
        if !response_trimmed.is_empty() {
            full_replacement.push_str(response_trimmed);
        }

        // 1. Drop everything from the audio turn start onwards.
        {
            let ctx = self.model.ctx_mut()?;
            // Remove [audio_start, +inf) for all sequences.
            let _ = ctx.clear_kv_cache_seq(None, Some(audio_start as u32), None);
        }
        self.state.n_past = audio_start;

        // 2. Re-decode the text-only replacement.
        let add_bos = if audio_start == 0 {
            AddBos::Always
        } else {
            AddBos::Never
        };
        let tokens = self
            .model
            .model()
            .str_to_token(&full_replacement, add_bos)
            .map_err(|e| LlmError::Tokenization(e.to_string()))?;

        {
            let ctx = self.model.ctx_mut()?;
            decode_tokens(ctx, &mut self.state, &tokens)?;
        }

        let n_past_after = self.state.n_past;
        self.pending_audio_start = None;

        Ok(AudioCompactionStats {
            n_past_before,
            n_past_after,
            tokens_freed: (n_past_before - n_past_after).max(0),
            compacted: true,
        })
    }

    /// Whether the KV cache was wiped to make room in the last response turn.
    pub fn was_context_cleaned(&self) -> bool {
        self.last_context_cleared
    }

    /// Current main-context usage snapshot (used, total, percent).
    pub fn context_usage_snapshot(&self) -> (u32, u32, f32) {
        let (used, total) = self.model.context_usage(self.state.n_past);
        let percent = if total > 0 {
            used as f32 / total as f32 * 100.0
        } else {
            0.0
        };
        (used, total, percent)
    }

    pub fn state(&self) -> &InferenceState {
        &self.state
    }
    pub fn model(&self) -> &LlmModel {
        &self.model
    }
    pub fn model_mut(&mut self) -> &mut LlmModel {
        &mut self.model
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Free functions
// ─────────────────────────────────────────────────────────────────────────────

fn ensure_context_space(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    n_ctx: u32,
    new_tokens: usize,
) -> Result<bool, AppError> {
    const MARGIN: i32 = 256;
    if state.n_past + new_tokens as i32 + MARGIN > n_ctx as i32 {
        ctx.clear_kv_cache();
        state.n_past = 0;
        state.system_prompt_evaluated = false;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn decode_tokens(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    tokens: &[LlamaToken],
) -> Result<(), AppError> {
    let last_index = tokens.len().saturating_sub(1);
    let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);

    for (i, &token) in tokens.iter().enumerate() {
        batch
            .add(token, state.n_past + i as i32, &[0], i == last_index)
            .map_err(|e| LlmError::Decode(e.to_string()))?;
    }

    ctx.decode(&mut batch)
        .map_err(|e| LlmError::Decode(e.to_string()))?;

    state.n_past += tokens.len() as i32;
    state.system_prompt_evaluated = true;
    Ok(())
}

/// Simplified streaming generator — no sentinel, no analysis split.
/// Emits TokenEvent::Token(text) for each generated token,
/// then TokenEvent::Done when EOG is reached.
fn generate_streaming<F>(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    config: &LlmConfig,
    inference_config: &dyn HasSamplerParams,
    mut on_token: F,
) -> Result<String, AppError>
where
    F: FnMut(TokenEvent),
{
    let model = ctx.model;

    let seed = resolve_seed(
        inference_config.seed_option(),
        inference_config.seed_value(),
    );

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_k(inference_config.top_k()),
        LlamaSampler::top_p(inference_config.top_p(), 1),
        LlamaSampler::temp(inference_config.temperature()),
        LlamaSampler::dist(seed),
    ]);

    let mut output_bytes = Vec::new();
    let mut batch = LlamaBatch::new(1, 1);

    for _ in 0..config.max_output_tokens {
        batch.clear();
        let new_token = sampler.sample(ctx, -1);
        sampler.accept(new_token);

        if new_token == model.token_eos() || model.is_eog_token(new_token) {
            on_token(TokenEvent::Done);
            break;
        }

        if let Ok(bytes) = model.token_to_piece_bytes(new_token, 256, true, None) {
            output_bytes.extend_from_slice(&bytes);
            let piece = String::from_utf8_lossy(&bytes).to_string();
            on_token(TokenEvent::Token(piece));
        }

        batch
            .add(new_token, state.n_past, &[0], true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        state.n_past += 1;
        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Decode(e.to_string()))?;
    }

    Ok(String::from_utf8_lossy(&output_bytes).trim().to_string())
}

// ── Trait to unify InferenceConfig and AnalysisConfig ────────

pub trait HasSamplerParams {
    fn top_k(&self) -> i32;
    fn top_p(&self) -> f32;
    fn temperature(&self) -> f32;
    fn seed_option(&self) -> SeedOption;
    fn seed_value(&self) -> u32;
}

impl HasSamplerParams for InferenceConfig {
    fn top_k(&self) -> i32 {
        self.top_k
    }
    fn top_p(&self) -> f32 {
        self.top_p
    }
    fn temperature(&self) -> f32 {
        self.temperature
    }
    fn seed_option(&self) -> SeedOption {
        self.seed_option.clone()
    }
    fn seed_value(&self) -> u32 {
        self.seed_value
    }
}

impl HasSamplerParams for AnalysisConfig {
    fn top_k(&self) -> i32 {
        self.top_k
    }
    fn top_p(&self) -> f32 {
        self.top_p
    }
    fn temperature(&self) -> f32 {
        self.temperature
    }
    fn seed_option(&self) -> SeedOption {
        self.seed_option.clone()
    }
    fn seed_value(&self) -> u32 {
        self.seed_value
    }
}

// ── Helpers ──────────────────────────────────────────────────

/// Build the text-only replacement for an audio turn prompt.
///
/// Replaces the MTMD audio marker with the transcription, preserving the
/// system prompt and turn framing. If the marker is absent (unexpected),
/// falls back to appending the transcription.
pub fn build_compacted_prompt(original_prompt: &str, transcription: &str) -> String {
    let marker = llama_cpp_2::mtmd::mtmd_default_marker();
    let transcription = transcription.trim();
    if original_prompt.contains(marker) {
        original_prompt.replacen(marker, transcription, 1)
    } else {
        format!("{original_prompt}\n{transcription}\n")
    }
}

fn resolve_seed(seed_option: SeedOption, seed_value: u32) -> u32 {
    match seed_option {
        SeedOption::Random => SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32,
        SeedOption::Fixed => seed_value,
    }
}

fn bos_flag(state: &InferenceState) -> AddBos {
    if !state.system_prompt_evaluated {
        AddBos::Always
    } else {
        AddBos::Never
    }
}

#[cfg(test)]
mod tests {
    use super::build_compacted_prompt;

    #[test]
    fn replaces_audio_marker_with_transcription() {
        let marker = llama_cpp_2::mtmd::mtmd_default_marker();
        let prompt = format!("<|turn>user\n\n{marker}\n<turn|>\n<|turn>model\n");
        let out = build_compacted_prompt(&prompt, "  I want to learn English  ");
        assert!(!out.contains(marker));
        assert!(out.contains("I want to learn English"));
        assert!(out.contains("<|turn>user"));
        assert!(out.contains("<|turn>model"));
    }

    #[test]
    fn preserves_system_prompt_when_compacting() {
        let marker = llama_cpp_2::mtmd::mtmd_default_marker();
        let prompt = format!("<|turn>system\nBe nice.\n<turn|>\n<|turn>user\n{marker}\n<turn|>\n<|turn>model\n");
        let out = build_compacted_prompt(&prompt, "hello");
        assert!(out.contains("Be nice."));
        assert!(out.contains("hello"));
        assert!(!out.contains(marker));
    }

    #[test]
    fn falls_back_when_no_marker_present() {
        let out = build_compacted_prompt("<|turn>user\nhi\n<turn|>\n", "hello");
        assert!(out.contains("hello"));
    }
}
