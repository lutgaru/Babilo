use llama_cpp_2::{
    context::params::LlamaContextParams,
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

static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

fn get_backend() -> &'static LlamaBackend {
    BACKEND.get_or_init(|| {
        let mut backend = LlamaBackend::init().expect("Failed to init llama backend");
        backend.void_logs(); // ← ¡Esto silencia los logs de llama.cpp!
        backend
    })
}

pub struct AudioLLM {
    #[allow(dead_code)]
    model: LlamaModel,
    ctx_params: LlamaContextParams,
    audio_embed_dim: usize,
    mtmd_context: Mutex<Option<MtmdContext>>,
}

impl AudioLLM {
    pub fn new(
        model_path: &Path,
        mmproj_path: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let backend = get_backend();

        let params = LlamaModelParams::default().with_n_gpu_layers(0);
        let model = LlamaModel::load_from_file(backend, model_path, &params)?;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(512)
            .with_n_ubatch(512)
            .with_n_seq_max(1);

        // ↓ Cargar mmproj aquí si se proporciona la ruta
        let mtmd_context = if let Some(mmproj) = mmproj_path {
            let params = llama_cpp_2::mtmd::MtmdContextParams::default();
            let ctx =
                MtmdContext::init_from_file(mmproj.to_string_lossy().as_ref(), &model, &params)
                    .map_err(|e| format!("Failed to init MTMD context: {}", e))?;
            Mutex::new(Some(ctx))
        } else {
            Mutex::new(None)
        };

        Ok(Self {
            model,
            ctx_params,
            audio_embed_dim: 2304,
            mtmd_context, // ← guardar el contexto cargado
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

    pub fn infer_audio(
        &mut self,
        audio_pcm: &[f32],
        // mmproj_path ya no es necesario aquí ↓
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use llama_cpp_2::mtmd::{mtmd_default_marker, MtmdBitmap, MtmdInputText};

        let backend = get_backend();

        // ↓ Obtener referencia al contexto ya cargado
        let mtmd_ctx_guard = self
            .mtmd_context
            .lock()
            .map_err(|e| format!("Poisoned lock: {}", e))?;

        let mtmd_ctx = mtmd_ctx_guard
            .as_ref()
            .ok_or("MtmdContext no inicializado — ¿olvidaste pasar mmproj_path en new()?")?;

        eprintln!("🔊 infer_audio START — {} samples", audio_pcm.len());

        // ── 2. Audio bitmap ───────────────────────────────────────────────────
        eprintln!(
            "  [2] Creando audio bitmap desde {} samples...",
            audio_pcm.len()
        );

        // Resamplear si el modelo espera 16kHz y cpal grabó a otra frecuencia
        // (cpal usa la frecuencia del sistema, puede ser 44100 o 48000)
        // Por ahora asumimos 16kHz — si no funciona hay que resamplear
        let audio_bitmap = MtmdBitmap::from_audio_data(audio_pcm)
            .map_err(|e| format!("Failed to create audio bitmap: {}", e))?;
        eprintln!(
            "  [2] ✅ Bitmap creado, is_audio={}",
            audio_bitmap.is_audio()
        );

        // ── 3. Prompt con instrucciones de sistema ─────────────────────────────────
        let marker = mtmd_default_marker();
        eprintln!("  [3] marker={:?}", marker);

        // 🔥 System prompt para controlar el estilo de respuesta
        let system_instruction = "You are a helpful assistant. Reply conversationally in 1-2 sentences max. Be concise and natural. No bullet points, no lists.";

        let full_prompt = format!(
            "<start_of_turn>user\n{}\n\n{}{}<end_of_turn>\n<start_of_turn>model\n",
            system_instruction, // ← Instrucciones de comportamiento
            marker,             // ← Placeholder para el audio
            if prompt.is_empty() {
                "".to_string()
            } else {
                format!("\n{}", prompt) // ← Prompt del usuario (si hay)
            }
        );

        eprintln!(
            "  [3] prompt completo: {:?}",
            &full_prompt[..full_prompt.len().min(120)]
        );

        // ── 4. Tokenizar ──────────────────────────────────────────────────────
        eprintln!("  [4] Tokenizando...");
        let input_text = MtmdInputText {
            text: full_prompt,
            add_special: true,
            parse_special: true,
        };
        let chunks = mtmd_ctx
            .tokenize(input_text, &[&audio_bitmap])
            .map_err(|e| format!("Tokenize failed: {}", e))?;
        eprintln!(
            "  [4] ✅ {} chunks, {} tokens totales, {} posiciones",
            chunks.len(),
            chunks.total_tokens(),
            chunks.total_positions()
        );

        // ── 5. LlamaContext ───────────────────────────────────────────────────
        eprintln!("  [5] Creando LlamaContext...");
        let mut ctx = self
            .model
            .new_context(backend, self.ctx_params.clone())
            .map_err(|e| format!("Failed to create context: {}", e))?;
        eprintln!("  [5] ✅ LlamaContext creado, n_ctx={}", ctx.n_ctx());

        // Verificar que el contexto es suficientemente grande
        if chunks.total_tokens() >= ctx.n_ctx() as usize {
            return Err(format!(
                "Audio demasiado largo: {} tokens > ctx {}",
                chunks.total_tokens(),
                ctx.n_ctx()
            )
            .into());
        }

        // ── 6. eval_chunks ────────────────────────────────────────────────────
        eprintln!("  [6] eval_chunks... (esto puede tardar)");

        const N_BATCH: i32 = 512;
        let n_past = chunks
            .eval_chunks(&mtmd_ctx, &ctx, 0, 0, N_BATCH, true) // true = request logits for last token
            .map_err(|e| format!("eval_chunks failed: {}", e))?;

        eprintln!("  [6] ✅ eval_chunks OK, n_past={}", n_past);

        if n_past == 0 {
            return Err("Error: n_past es 0, la evaluación no avanzó".into());
        }

        // 🔥 FIX CRÍTICO: "Bridge decode" para asegurar logits en índice 0
        // eval_chunks puede dejar los logits en un índice impredecible.
        // Decodificamos un token neutro en posición n_past con logits=true
        // para resetear el buffer y garantizar que el primer sample lea desde idx 0.
        eprintln!("  [6.5] 🔗 Bridge decode para preparar logits...");
        let mut bridge_batch = LlamaBatch::new(1, 1);
        // Usamos token_bos como placeholder; NO usaremos su output para texto
        bridge_batch
            .add(self.model.token_bos(), n_past as i32, &[0], true) // ← logits=true es vital
            .map_err(|e| format!("Bridge batch failed: {}", e))?;
        ctx.decode(&mut bridge_batch)
            .map_err(|e| format!("Bridge decode failed: {}", e))?;
        // ✅ Ahora los logits están garantizados en batch index 0

        // ── 7. Generación ─────────────────────────────────────────────────────
        eprintln!("  [7] Iniciando generación...");

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.7),
            LlamaSampler::dist(42),
        ]);

        let mut output = String::new();
        // ⚠️ n_cur empieza en n_past + 1 porque el bridge token ocupa posición n_past
        let mut n_cur = n_past + 1;

        for i in 0..150 {
            // ✅ Siempre sampleamos desde índice 0 (logits garantizados tras bridge)
            let new_token = sampler.sample(&ctx, 0);
            sampler.accept(new_token);

            let is_eos = new_token == self.model.token_eos();
            let is_eog = self.model.is_eog_token(new_token);

            if is_eos || is_eog {
                eprintln!("  [7] ⛔ Stop token en iteración {}", i);
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

            // Preparar batch para SIGUIENTE decode: marcar ESTE token para logits
            let mut batch = LlamaBatch::new(1, 1);
            batch
                .add(new_token, n_cur, &[0], true) // ← logits=true para próxima iteración
                .map_err(|e| format!("Batch add failed: {}", e))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| format!("Decode failed: {}", e))?;
            // ✅ Tras decode, logits del nuevo token están en índice 0
        }

        eprintln!("🔊 infer_audio END — output={:?}", output);
        Ok(output.trim().to_string())
    }

    pub fn models_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }
}
