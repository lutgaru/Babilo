use llama_cpp_2::{
    context::params::LlamaContextParams,
    context::LlamaContext,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel},
    mtmd::MtmdContext,
    sampling::LlamaSampler, // ← added
};
use ndarray::Array2;
use std::path::Path;
use std::sync::OnceLock;
use std::{num::NonZeroU32, sync::Mutex};

// ── Helper seguro para lifetime erasure ─────────────────────────────
// Safe porque: AudioLLM posee model y ctx, y model nunca se mueve/droppea antes que ctx
fn erase_ctx_lifetime<'a>(ctx: LlamaContext<'a>) -> LlamaContext<'static> {
    unsafe { std::mem::transmute(ctx) }
}

fn restore_ctx_lifetime<'a>(ctx: &'a mut LlamaContext<'static>) -> &'a mut LlamaContext<'a> {
    unsafe { std::mem::transmute(ctx) }
}

static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

fn get_backend() -> &'static LlamaBackend {
    BACKEND.get_or_init(|| {
        let mut backend = LlamaBackend::init().expect("Failed to init llama backend");
        backend.void_logs(); // ← ¡Esto silencia los logs de llama.cpp!
        backend
    })
}

struct SendableContext(LlamaContext<'static>);

unsafe impl Send for SendableContext {}
unsafe impl Sync for SendableContext {}

pub struct AudioLLM {
    model: LlamaModel,
    ctx_params: LlamaContextParams,
    audio_embed_dim: usize,
    mtmd_context: Mutex<Option<MtmdContext>>,
    ctx: Mutex<Option<SendableContext>>, // ← wrapped
    n_past: Mutex<i32>,
    system_prompt_evaluated: Mutex<bool>,
}

impl AudioLLM {
    pub fn new(
        model_path: &Path,
        mmproj_path: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backend = get_backend();

        // ── 1. ACTIVATE GPU ────────────────────────────────────
        // Change 0 to 99 to offload all layers to GPU
        let params = LlamaModelParams::default().with_n_gpu_layers(99);

        let model = LlamaModel::load_from_file(backend, model_path, &params)?;

        // ── 2. CONFIGURE AUDIO CONTEXT ───────────────────────────
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(2048) // Aumentamos batch para que la GPU procese audio más rápido
            .with_n_ubatch(512)
            .with_n_seq_max(1)
            .with_embeddings(true); // <--- VITAL para que el proyector de audio funcione en GPU

        // ── 3. LOAD MMPROJ ───────────────────────────────────────────────
        let mtmd_context = if let Some(mmproj) = mmproj_path {
            let params = llama_cpp_2::mtmd::MtmdContextParams::default();
            let ctx =
                MtmdContext::init_from_file(mmproj.to_string_lossy().as_ref(), &model, &params)
                    .map_err(|e| format!("Failed to init MTMD context: {}", e))?;
            Mutex::new(Some(ctx))
        } else {
            Mutex::new(None)
        };

        // 🔥 Lifetime erasure al crear el contexto
        let ctx_raw = model.new_context(backend, ctx_params.clone())?;
        let ctx_static = erase_ctx_lifetime(ctx_raw);

        Ok(Self {
            model,
            ctx_params,
            audio_embed_dim: 2304,
            mtmd_context,
            ctx: Mutex::new(Some(SendableContext(ctx_static))),
            n_past: Mutex::new(0),
            system_prompt_evaluated: Mutex::new(false),
        })
    }

    pub fn reset_context(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let backend = get_backend();

        let mut ctx_guard = self
            .ctx
            .lock()
            .map_err(|_| "Poisoned lock in reset_context")?;
        let mut n_past_guard = self
            .n_past
            .lock()
            .map_err(|_| "Poisoned lock in reset_context")?;
        let mut system_flag_guard = self
            .system_prompt_evaluated
            .lock()
            .map_err(|_| "Poisoned lock in reset_context")?;

        // ── Dropear el contexto viejo ANTES de crear el nuevo ─────────────────
        // Evita tener dos contextos en memoria al mismo tiempo
        *ctx_guard = None;

        // ── Crear contexto nuevo ──────────────────────────────────────────────
        let ctx_raw = self.model.new_context(backend, self.ctx_params.clone())?;
        *ctx_guard = Some(SendableContext(erase_ctx_lifetime(ctx_raw)));

        *n_past_guard = 0;
        *system_flag_guard = false;

        eprintln!("🔄 Contexto reseteado - nueva conversación iniciada");
        Ok(())
    }

    pub fn context_usage(&self) -> (i32, i32) {
        let n_past = self.n_past.lock().unwrap_or_else(|e| e.into_inner());
        let n_ctx = self
            .ctx_params
            .n_ctx()
            .map(|n| n.get() as i32)
            .unwrap_or(4096);
        (*n_past, n_ctx)
    }

    pub fn context_is_full(&self) -> bool {
        let (n_past, n_ctx) = self.context_usage();
        n_past + 256 > n_ctx
    }

    fn slide_window_if_needed(
        &mut self,
        ctx: &mut LlamaContext<'_>,
        n_past: &mut i32,
        new_tokens: usize,
    ) -> Result<(), String> {
        //TODO
        Ok(())
    }

    pub fn infer(
        &mut self,
        _mel_features: Vec<Array2<f32>>,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut ctx_guard = self.ctx.lock().map_err(|_| "Poisoned lock in infer")?;
        let ctx_static = ctx_guard.as_mut().ok_or("Contexto no inicializado")?;
        let ctx = restore_ctx_lifetime(&mut ctx_static.0);

        let mut n_past_guard = self.n_past.lock().map_err(|_| "Poisoned lock in infer")?;
        let mut system_flag_guard = self
            .system_prompt_evaluated
            .lock()
            .map_err(|_| "Poisoned lock in infer")?;

        eprintln!(
            "📍 infer START — n_past={}, system_flag={}",
            *n_past_guard, *system_flag_guard
        );

        // ── Prompt: solo incluye system en el primer turno ────────────────
        let full_prompt = if !*system_flag_guard {
            format!(
            "<start_of_turn>user\nYou are a helpful assistant. Reply conversationally in 1-2 sentences max. Be concise and natural. No bullet points, no lists.\n\n{}<end_of_turn>\n<start_of_turn>model\n",
            prompt
        )
        } else {
            format!(
                "<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n",
                prompt
            )
        };

        let add_bos = if !*system_flag_guard {
            AddBos::Always
        } else {
            AddBos::Never
        };

        let tokens_list = self
            .model
            .str_to_token(&full_prompt, add_bos)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        // ── Verificar espacio ─────────────────────────────────────────────
        let n_ctx = ctx.n_ctx() as i32;
        if *n_past_guard + tokens_list.len() as i32 + 256 > n_ctx {
            eprintln!("🪟 Contexto lleno, reseteando KV cache");
            ctx.clear_kv_cache();
            *n_past_guard = 0;
            *system_flag_guard = false;

            // Re-tokenizar con system prompt ya que reseteamos
            let full_prompt = format!(
            "<start_of_turn>user\nYou are a helpful assistant. Reply conversationally in 1-2 sentences max. Be concise and natural. No bullet points, no lists.\n\n{}<end_of_turn>\n<start_of_turn>model\n",
            prompt
        );
            let tokens_list = self
                .model
                .str_to_token(&full_prompt, AddBos::Always)
                .map_err(|e| format!("Tokenization failed after reset: {}", e))?;

            let last_index = tokens_list.len().saturating_sub(1);
            let token_len = tokens_list.len();
            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            for (i, token) in tokens_list.into_iter().enumerate() {
                batch
                    .add(token, i as i32, &[0], i == last_index)
                    .map_err(|e| format!("Batch add failed: {}", e))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| format!("Prompt decode failed: {}", e))?;
            *n_past_guard = token_len as i32;
            *system_flag_guard = true;
        } else {
            // ── Caso normal: decodificar desde n_past actual ──────────────
            let last_index = tokens_list.len().saturating_sub(1);
            let token_len = tokens_list.len();
            let mut batch = LlamaBatch::new(ctx.n_ctx() as usize, 1);
            for (i, token) in tokens_list.into_iter().enumerate() {
                let pos = *n_past_guard + i as i32;
                batch
                    .add(token, pos, &[0], i == last_index)
                    .map_err(|e| format!("Batch add failed: {}", e))?;
            }
            ctx.decode(&mut batch)
                .map_err(|e| format!("Prompt decode failed: {}", e))?;
            *n_past_guard += token_len as i32;
            *system_flag_guard = true;
        }

        // ── Generación ────────────────────────────────────────────────────
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.7),
            LlamaSampler::dist(42),
        ]);

        let mut output = String::new();

        for _ in 0..150 {
            let new_token = sampler.sample(ctx, -1); // ← -1 = último logit
            sampler.accept(new_token);

            if new_token == self.model.token_eos() || self.model.is_eog_token(new_token) {
                break;
            }

            if let Ok(bytes) = self.model.token_to_piece_bytes(new_token, 256, true, None) {
                let piece = String::from_utf8_lossy(&bytes).to_string();
                if piece.contains("<end_of_turn>") || piece.contains("<start_of_turn>") {
                    break;
                }
                output.push_str(&piece);
                if output.len() > 150
                    && (piece.contains('.') || piece.contains('!') || piece.contains('?'))
                {
                    break;
                }
            }

            let mut gen_batch = LlamaBatch::new(1, 1);
            gen_batch
                .add(new_token, *n_past_guard, &[0], true)
                .map_err(|e| format!("Batch add (gen) failed: {}", e))?;
            *n_past_guard += 1;
            ctx.decode(&mut gen_batch)
                .map_err(|e| format!("Token decode failed: {}", e))?;
        }

        eprintln!(
            "📍 infer END — n_past={}, output_len={}",
            *n_past_guard,
            output.len()
        );
        Ok(output.trim().to_string())
    }

    pub fn infer_audio(
        &mut self,
        audio_pcm: &[f32],
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use llama_cpp_2::mtmd::{mtmd_default_marker, MtmdBitmap, MtmdInputText};

        let mtmd_ctx_guard = self
            .mtmd_context
            .lock()
            .map_err(|e| format!("Poisoned lock mtmd: {}", e))?;
        let mtmd_ctx = mtmd_ctx_guard
            .as_ref()
            .ok_or("MtmdContext no inicializado — ¿olvidaste pasar mmproj_path en new()?")?;

        let mut ctx_guard = self
            .ctx
            .lock()
            .map_err(|_| "Poisoned lock in infer_audio")?;
        let ctx_static = ctx_guard.as_mut().ok_or("Contexto Llama no inicializado")?;
        let ctx = restore_ctx_lifetime(&mut ctx_static.0);

        let mut n_past_guard = self
            .n_past
            .lock()
            .map_err(|_| "Poisoned lock in infer_audio")?;
        let mut system_flag_guard = self
            .system_prompt_evaluated
            .lock()
            .map_err(|_| "Poisoned lock in infer_audio")?;

        eprintln!(
            "🔊 infer_audio START — {} samples, n_past={}",
            audio_pcm.len(),
            *n_past_guard
        );

        let audio_bitmap = MtmdBitmap::from_audio_data(audio_pcm)
            .map_err(|e| format!("Failed to create audio bitmap: {}", e))?;

        let marker = mtmd_default_marker();
        let system_instruction = "You are a helpful assistant. Reply conversationally in 1-2 sentences max. Be concise and natural. No bullet points, no lists.";

        let full_prompt = if !*system_flag_guard {
            format!(
                "<start_of_turn>user\n{}\n\n{}{}<end_of_turn>\n<start_of_turn>model\n",
                system_instruction,
                marker,
                if prompt.is_empty() {
                    "".to_string()
                } else {
                    format!("\n{}", prompt)
                }
            )
        } else {
            format!(
                "<start_of_turn>user\n{}{}<end_of_turn>\n<start_of_turn>model\n",
                marker,
                if prompt.is_empty() {
                    "".to_string()
                } else {
                    format!("\n{}", prompt)
                }
            )
        };

        let input_text = MtmdInputText {
            text: full_prompt,
            add_special: !*system_flag_guard,
            parse_special: true,
        };

        let chunks = mtmd_ctx
            .tokenize(input_text, &[&audio_bitmap])
            .map_err(|e| format!("Tokenize failed: {}", e))?;

        eprintln!(
            "  [chunks] {} chunks, {} tokens, {} posiciones",
            chunks.len(),
            chunks.total_tokens(),
            chunks.total_positions()
        );

        // ── Verificar espacio ─────────────────────────────────────────────
        let n_ctx = ctx.n_ctx() as i32;
        if *n_past_guard + chunks.total_tokens() as i32 + 256 > n_ctx {
            eprintln!("🪟 Contexto lleno, reseteando KV cache");
            ctx.clear_kv_cache();
            *n_past_guard = 0;
            *system_flag_guard = false;
        }

        // ── eval_chunks: encode audio + decode prompt ─────────────────────
        let eval_n_past = chunks
            .eval_chunks(mtmd_ctx, ctx, *n_past_guard, 0, 512, true)
            .map_err(|e| format!("eval_chunks failed: {}", e))?;

        eprintln!("  [eval] n_past: {} → {}", *n_past_guard, eval_n_past);
        *n_past_guard = eval_n_past;
        *system_flag_guard = true;

        // ── Generación ────────────────────────────────────────────────────
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.7),
            LlamaSampler::dist(42),
        ]);

        let mut output = String::new();

        for i in 0..150 {
            let new_token = sampler.sample(ctx, -1); // ← -1 = último logit
            sampler.accept(new_token);

            if new_token == self.model.token_eos() || self.model.is_eog_token(new_token) {
                eprintln!("  ⛔ Stop token en iteración {}", i);
                break;
            }

            if let Ok(bytes) = self.model.token_to_piece_bytes(new_token, 256, true, None) {
                let piece = String::from_utf8_lossy(&bytes).to_string();
                if piece.contains("<end_of_turn>") || piece.contains("<start_of_turn>") {
                    break;
                }
                output.push_str(&piece);
                if output.len() > 150
                    && (piece.contains('.') || piece.contains('!') || piece.contains('?'))
                {
                    break;
                }
            }

            let mut batch = LlamaBatch::new(1, 1);
            batch
                .add(new_token, *n_past_guard, &[0], true)
                .map_err(|e| format!("Batch add failed: {}", e))?;
            *n_past_guard += 1;
            ctx.decode(&mut batch)
                .map_err(|e| format!("Decode failed: {}", e))?;
        }

        eprintln!(
            "🔊 infer_audio END — n_past={}, output={:?}",
            *n_past_guard, output
        );
        Ok(output
            .trim()
            .trim_end_matches("<start_of_turn>")
            .trim_end_matches("<end_of_turn>")
            .trim()
            .to_string())
    }

    pub fn models_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }
}
