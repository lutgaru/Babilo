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
    config::{InferenceConfig, LlmConfig, SeedOption},
    errors::{AppError, LlmError},
    llama::model::LlmModel,
    schemas::{TokenEvent, SENTINEL},
};
use llama_cpp_2::{
    context::LlamaContext,
    llama_batch::LlamaBatch,
    model::AddBos,
    mtmd::{MtmdBitmap, MtmdInputText},
    sampling::LlamaSampler,
    token::LlamaToken,
};

// ─────────────────────────────────────────────────────────────────────────────
// Estado
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct InferenceState {
    pub n_past: i32,
    pub system_prompt_evaluated: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Motor
// ─────────────────────────────────────────────────────────────────────────────

pub struct InferenceEngine {
    model: LlmModel,
    state: InferenceState,
}

impl InferenceEngine {
    pub fn new(model: LlmModel) -> Self {
        Self {
            model,
            state: InferenceState::default(),
        }
    }

    // ── API pública ───────────────────────────────────────────────────────────

    // inference.rs — fix infer_audio_streaming: eval_chunks + closure callback
    pub fn infer_audio_streaming(
        &mut self,
        audio_pcm: &[f32],
        prompt: &str,
        on_token: impl FnMut(TokenEvent), // ← closure, not Sender
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

        ensure_context_space(ctx, &mut self.state, n_ctx, total_tokens)?;

        // ← was missing in the broken version
        let new_n_past = chunks
            .eval_chunks(mtmd, ctx, self.state.n_past, 0, 512, true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        self.state.n_past = new_n_past;
        self.state.system_prompt_evaluated = true;

        generate_babilo_streaming(ctx, &mut self.state, &config, &inference_config, on_token)?;

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

        ensure_context_space(ctx, &mut self.state, n_ctx, tokens.len())?;
        decode_tokens(ctx, &mut self.state, &tokens)?;

        self.state.system_prompt_evaluated = true;

        generate_babilo_streaming(ctx, &mut self.state, &config, &inference_config, on_token)?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), AppError> {
        self.model.reset_context()?;
        self.state = InferenceState::default();
        Ok(())
    }

    pub fn state(&self) -> &InferenceState {
        &self.state
    }
    pub fn model(&self) -> &LlmModel {
        &self.model
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Funciones libres — reciben componentes separados para evitar borrow conflicts
// ─────────────────────────────────────────────────────────────────────────────

/// Limpia KV cache si el contexto está lleno.
fn ensure_context_space(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    n_ctx: u32,
    new_tokens: usize,
) -> Result<(), AppError> {
    const MARGIN: i32 = 256;
    eprintln!(
        "Checking context space: n_past={}, new_tokens={}, n_ctx={}, percent_used={:.2}%",
        state.n_past, new_tokens, n_ctx, (state.n_past + new_tokens as i32) as f64 / n_ctx as f64 * 100.0
    );
    if state.n_past + new_tokens as i32 + MARGIN > n_ctx as i32 {
        ctx.clear_kv_cache();
        state.n_past = 0;
        state.system_prompt_evaluated = false;
    }
    Ok(())
}

/// Decodifica un bloque de tokens prompt en el contexto.
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

//generate a especific streaming response, with detection of the sentinel to separate response from analysis.
fn generate_babilo_streaming<F>(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    config: &LlmConfig,
    inference_config: &InferenceConfig,
    mut on_token: F,
) -> Result<String, AppError>
where
    F: FnMut(TokenEvent),
{
    let model = ctx.model;

    let seed = resolve_seed(inference_config);

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_k(inference_config.top_k),
        LlamaSampler::top_p(inference_config.top_p, 1),
        LlamaSampler::temp(inference_config.temperature),
        LlamaSampler::dist(seed),
    ]);

    let mut output_bytes = Vec::new();
    let mut sentinel_buf = String::new(); // lookahead buffer for sentinel detection
    let mut sentinel_found = false;
    let mut batch = LlamaBatch::new(1, 1);

    for _ in 0..config.max_output_tokens {
        batch.clear();
        let new_token = sampler.sample(ctx, -1);
        sampler.accept(new_token);

        if new_token == model.token_eos() || model.is_eog_token(new_token) {
            on_token(TokenEvent::Done);
            eprintln!("EOG token reached, stopping generation.");
            break;
        }

        if let Ok(bytes) = model.token_to_piece_bytes(new_token, 256, true, None) {
            let piece = String::from_utf8_lossy(&bytes).to_string();
            output_bytes.extend_from_slice(&bytes);

            if sentinel_found {
                // ── Analysis phase ────────────────────────────────────────
                on_token(TokenEvent::AnalysisToken(piece));
            } else {
                // ── Response phase: watch for sentinel ────────────────────
                sentinel_buf.push_str(&piece);

                if sentinel_buf.contains(SENTINEL) {
                    // Split: flush whatever came before sentinel to TTS
                    let (before, _) = sentinel_buf.split_once(SENTINEL).unwrap();
                    if !before.is_empty() {
                        on_token(TokenEvent::ResponseToken(before.to_string()));
                    }
                    on_token(TokenEvent::SentinelReached);
                    sentinel_found = true;
                    sentinel_buf.clear();
                } else if sentinel_buf.len() > SENTINEL.len() {
                    let mut safe_len = sentinel_buf.len() - SENTINEL.len();
                    // retroceder hasta encontrar un límite válido
                    while safe_len > 0 && !sentinel_buf.is_char_boundary(safe_len) {
                        safe_len -= 1;
                    }

                    if safe_len > 0 {
                        let flushed = sentinel_buf[..safe_len].to_string();
                        sentinel_buf = sentinel_buf[safe_len..].to_string();
                        on_token(TokenEvent::ResponseToken(flushed));
                    }
                }
                // else: keep buffering (potential partial sentinel match)
            }
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

/// Genera tokens de respuesta hasta EOG o límite de config.
/// Obtiene el LlamaModel directamente del contexto (ctx.model()),
/// así nunca necesita un &LlmModel extra.
fn resolve_seed(inference_config: &InferenceConfig) -> u32 {
    match inference_config.seed_option {
        SeedOption::Random => SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32,
        SeedOption::Fixed => inference_config.seed_value,
    }
}

fn bos_flag(state: &InferenceState) -> AddBos {
    if !state.system_prompt_evaluated {
        AddBos::Always
    } else {
        AddBos::Never
    }
}


