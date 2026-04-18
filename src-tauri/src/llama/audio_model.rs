use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel}, // ← removed Special
    sampling::LlamaSampler,                                // ← added
};
use ndarray::Array2;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::OnceLock;

static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

fn get_backend() -> &'static LlamaBackend {
    BACKEND.get_or_init(|| LlamaBackend::init().expect("Failed to init llama backend"))
}

pub struct AudioLLM {
    #[allow(dead_code)]
    model: LlamaModel,
    ctx_params: LlamaContextParams,
    audio_embed_dim: usize,
}

impl AudioLLM {
    pub fn new(model_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let backend = get_backend();

        let params = LlamaModelParams::default().with_n_gpu_layers(0);

        let model = LlamaModel::load_from_file(backend, model_path, &params)?;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(512)
            .with_n_ubatch(512)
            .with_n_seq_max(1);

        Ok(Self {
            model,
            ctx_params,
            audio_embed_dim: 2304,
        })
    }

    pub fn infer(
        &mut self,
        _mel_features: Vec<Array2<f32>>,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let backend = get_backend();

        // ── System prompt: respuestas cortas y conversacionales ───────────────
        let full_prompt = format!(
            "<start_of_turn>user\nYou are a helpful assistant. Reply conversationally in 1-2 sentences max. Be concise and natural. No bullet points, no lists.\n\n{}<end_of_turn>\n<start_of_turn>model\n",
            prompt
        );

        let mut ctx = self
            .model
            .new_context(backend, self.ctx_params.clone())
            .map_err(|e| format!("Failed to create context: {}", e))?;

        let tokens_list = self
            .model
            .str_to_token(&full_prompt, AddBos::Always)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        let n_ctx = ctx.n_ctx() as usize;
        if tokens_list.len() >= n_ctx {
            return Err(format!("Prompt too long: {} tokens", tokens_list.len()).into());
        }

        let last_index = tokens_list.len() - 1;
        let mut batch = LlamaBatch::new(n_ctx, 1);
        for (i, token) in tokens_list.into_iter().enumerate() {
            batch
                .add(token, i as i32, &[0], i == last_index)
                .map_err(|e| format!("Batch add failed: {}", e))?;
        }
        ctx.decode(&mut batch)
            .map_err(|e| format!("Prompt decode failed: {}", e))?;

        // ── Sampler: top-k + temperatura baja = respuestas naturales pero concisas
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.7),
            LlamaSampler::dist(42), // seed fijo para reproducibilidad
        ]);

        let mut output = String::new();
        let max_new_tokens = 150; // ~200 caracteres aprox
        let mut n_cur = batch.n_tokens() as i32;

        for _ in 0..max_new_tokens {
            let new_token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(new_token);

            if new_token == self.model.token_eos() || self.model.is_eog_token(new_token) {
                break;
            }

            // Parar también si el modelo empieza a escribir el próximo turno
            if let Ok(bytes) = self.model.token_to_piece_bytes(new_token, 256, true, None) {
                let piece = String::from_utf8_lossy(&bytes).to_string();

                // Gemma usa <end_of_turn> para terminar
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

            batch.clear();
            batch
                .add(new_token, n_cur, &[0], true)
                .map_err(|e| format!("Batch add (gen) failed: {}", e))?;
            n_cur += 1;
            ctx.decode(&mut batch)
                .map_err(|e| format!("Token decode failed: {}", e))?;
        }

        Ok(output.trim().to_string())
    }

    pub fn models_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }
}
