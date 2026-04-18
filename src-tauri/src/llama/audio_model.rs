// ✅ Imports corregidos para llama-cpp-2 v0.1.143
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend, // ✅ LlamaBackend está en llama_backend
    model::{params::LlamaModelParams, LlamaModel},
    token::LlamaToken, // ✅ LlamaToken es público en token (no en token::data)
};
use ndarray::Array2;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::OnceLock;

// ✅ Backend global con tipo correcto
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

        // ✅ Cargar modelo con parámetros
        let params = LlamaModelParams::default().with_n_gpu_layers(0); // 🔧 Ajustar según tu hardware

        let model = LlamaModel::load_from_file(backend, model_path, &params)?;

        // ✅ with_n_ctx espera Option<NonZeroU32>
        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(4096));

        Ok(Self {
            model,
            ctx_params,
            audio_embed_dim: 2304, // Gemma 4 audio embed dim (referencia)
        })
    }

    /// Procesa features de audio → respuesta de texto
    /// 🔧 NOTA: llama.cpp aún NO tiene soporte nativo para Gemma 4 audio
    /// Esto es un placeholder hasta que se implemente el encoder Conformer
    pub fn infer(
        &mut self,
        _mel_features: Vec<Array2<f32>>,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 🔧 PLACEHOLDER: Usar inferencia de texto normal por ahora
        // Cuando llama-cpp-2 soporte audio nativo, reemplazar con:
        // - Codificar mel_features a embeddings
        // - Inyectar en el contexto del modelo
        // - Generar tokens de respuesta

        Ok(format!("[Echo] {}", prompt))

        /*
        // 🔮 CÓDIGO FUTURO (cuando haya soporte):
        use llama_cpp_2::context::LlamaContext;

        let ctx = self.model.new_context(&self.ctx_params)?;

        // Tokenizar prompt
        let tokens: Vec<LlamaToken> = ctx.tokenize(prompt.as_bytes(), true)?;

        // Evaluar contexto
        ctx.eval(&tokens)?;

        // Generar respuesta
        let mut output = String::new();
        for _ in 0..256 {
            let token = ctx.sample_token()?;
            if token == self.model.token_eos() { break; }
            if let Some(text) = self.model.token_to_str(token, &ctx)? {
                output.push_str(&text);
            }
        }
        Ok(output)
        */
    }

    pub fn models_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }
}
