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

use crate::{
    config::LlmConfig,
    errors::{AppError, LlmError},
    llama::model::LlmModel,
};
use llama_cpp_2::{
    context::LlamaContext,
    llama_batch::LlamaBatch,
    model::{AddBos},
    mtmd::{mtmd_default_marker, MtmdBitmap, MtmdInputText},
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

    pub fn infer_text(&mut self, prompt: &str) -> Result<String, AppError> {
        // ── Fase 1: extraer todo de `model` ANTES de ctx_mut ─────────────────
        // Una vez que llamemos ctx_mut(), el borrow checker ve `self.model`
        // como mutuamente prestado y ya no permite acceder a ningún otro campo.
        let full_prompt = build_text_prompt(prompt, &self.state);
        let add_bos = bos_flag(&self.state);

        let tokens = self
            .model
            .model()
            .str_to_token(&full_prompt, add_bos)
            .map_err(|e| LlmError::Tokenization(e.to_string()))?;

        // Copiamos/clonamos lo que necesitaremos después del borrow mutable.
        // n_ctx es u32 (Copy) y config se clona una vez; ambos son baratos.
        let n_ctx = self.model.n_ctx(); // u32, Copy
        let config = self.model.config().clone();

        // ── Fase 2: borrow mutable exclusivo ─────────────────────────────────
        let ctx = self.model.ctx_mut()?;

        ensure_context_space(ctx, &mut self.state, n_ctx, tokens.len())?;
        decode_tokens(ctx, &mut self.state, &tokens)?;
        generate(ctx, &mut self.state, &config)
    }

    pub fn infer_audio(&mut self, audio_pcm: &[f32], prompt: &str) -> Result<String, AppError> {
        // ── Fase 1: todo antes de tocar ctx ──────────────────────────────────
        let full_prompt = build_audio_prompt(prompt, &self.state);
        let add_special = !self.state.system_prompt_evaluated;

        let audio_bitmap = MtmdBitmap::from_audio_data(audio_pcm)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        // tokenize necesita &mtmd (inmutable). Lo hacemos en bloque propio
        // para que ese borrow quede suelto antes de split_ctx_mtmd.
        let chunks = {
            let mtmd = self
                .model
                .mtmd_context()
                .ok_or(LlmError::MtmdInit("No mmproj loaded".into()))?;

            mtmd.tokenize(
                MtmdInputText {
                    text: full_prompt,
                    add_special,
                    parse_special: true,
                },
                &[&audio_bitmap],
            )
            .map_err(|e| LlmError::Tokenization(e.to_string()))?
        }; // ← borrow de mtmd_context termina aquí

        let total_tokens = chunks.total_tokens();
        let n_ctx = self.model.n_ctx();
        let config = self.model.config().clone();

        // ── Fase 2: split de borrows ──────────────────────────────────────────
        // eval_chunks necesita (&mut ctx, &mtmd) simultáneamente.
        // Como son campos distintos del struct, split_ctx_mtmd() los retorna
        // juntos usando punteros raw internamente — seguro y sin transmute aquí.
        let (ctx, mtmd) = self.model.split_ctx_mtmd()?;

        ensure_context_space(ctx, &mut self.state, n_ctx, total_tokens)?;

        let new_n_past = chunks
            .eval_chunks(mtmd, ctx, self.state.n_past, 0, 512, true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        self.state.n_past = new_n_past;
        self.state.system_prompt_evaluated = true;

        generate(ctx, &mut self.state, &config)
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

/// Genera tokens de respuesta hasta EOG o límite de config.
/// Obtiene el LlamaModel directamente del contexto (ctx.model()),
/// así nunca necesita un &LlmModel extra.
fn generate(
    ctx: &mut LlamaContext<'_>,
    state: &mut InferenceState,
    config: &LlmConfig,
) -> Result<String, AppError> {
    // LlamaContext expone model() → &LlamaModel. Este borrow es del ctx,
    // completamente independiente del LlmModel wrapper.
    let model = ctx.model;

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::top_k(40),
        LlamaSampler::top_p(0.9, 1),
        LlamaSampler::temp(0.7),
        LlamaSampler::dist(42),
    ]);

    let mut output = String::new();

    for _ in 0..config.max_output_tokens {
        let new_token = sampler.sample(ctx, -1);
        sampler.accept(new_token);

        if new_token == model.token_eos() || model.is_eog_token(new_token) {
            break;
        }

        if let Ok(bytes) = model.token_to_piece_bytes(new_token, 256, true, None) {
            let piece = String::from_utf8_lossy(&bytes);

            if piece.contains("<end_of_turn>") || piece.contains("<start_of_turn>") {
                break;
            }

            output.push_str(&piece);

            if output.len() > 150 && piece.contains(|c: char| c == '.' || c == '!' || c == '?') {
                break;
            }
        }

        let mut batch = LlamaBatch::new(1, 1);
        batch
            .add(new_token, state.n_past, &[0], true)
            .map_err(|e| LlmError::Decode(e.to_string()))?;

        state.n_past += 1;
        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Decode(e.to_string()))?;
    }

    Ok(output.trim().to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers de prompt
// ─────────────────────────────────────────────────────────────────────────────

fn system_instruction() -> &'static str {
    "You are a helpful assistant. Reply conversationally in 1-2 sentences max. \
     Be concise and natural. No bullet points, no lists."
}

fn bos_flag(state: &InferenceState) -> AddBos {
    if !state.system_prompt_evaluated {
        AddBos::Always
    } else {
        AddBos::Never
    }
}

fn build_text_prompt(prompt: &str, state: &InferenceState) -> String {
    if !state.system_prompt_evaluated {
        format!(
            "<start_of_turn>user\n{}\n\n{}<end_of_turn>\n<start_of_turn>model\n",
            system_instruction(),
            prompt,
        )
    } else {
        format!(
            "<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n",
            prompt,
        )
    }
}

fn build_audio_prompt(prompt: &str, state: &InferenceState) -> String {
    let marker = mtmd_default_marker();
    let suffix = if prompt.is_empty() {
        String::new()
    } else {
        format!("\n{}", prompt)
    };

    if !state.system_prompt_evaluated {
        format!(
            "<start_of_turn>user\n{}\n\n{}{}<end_of_turn>\n<start_of_turn>model\n",
            system_instruction(),
            marker,
            suffix,
        )
    } else {
        format!(
            "<start_of_turn>user\n{}{}<end_of_turn>\n<start_of_turn>model\n",
            marker, suffix,
        )
    }
}
