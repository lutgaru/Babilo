//! Gestión del modelo llama.cpp: carga, contexto, configuración

use llama_cpp_2::{
    context::params::LlamaContextParams,
    context::LlamaContext,
    llama_backend::LlamaBackend,
    model::{params::LlamaModelParams, LlamaModel},
    mtmd::MtmdContext,
};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use crate::{config::LlmConfig, errors::{AppError, LlmError}};

static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

pub fn get_backend() -> &'static LlamaBackend {
    BACKEND.get_or_init(|| {
        let mut backend = LlamaBackend::init().expect("Failed to init llama backend");
        backend.void_logs();
        backend
    })
}

/// Modelo y contexto LLM.
///
/// # Orden de campos (crítico para drop order)
///
/// Rust dropea los campos en orden INVERSO al de declaración.
/// `ctx` debe declararse ANTES de `model` para que sea dropeado primero,
/// garantizando que el contexto no outlive al modelo que referencia.
///
/// # Safety del transmute
///
/// `LlamaContext<'a>` toma prestado de `LlamaModel`. Usamos `'static` como
/// lifetime erased porque la crate no expone un constructor que permita
/// self-referential structs. El invariante es: mientras `LlmModel` viva,
/// `model` nunca se mueve ni se dropea antes que `ctx`.
pub struct LlmModel {
    // IMPORTANTE: ctx antes de model → se dropea primero
    ctx:          Option<LlamaContext<'static>>,
    model:        LlamaModel,
    ctx_params:   LlamaContextParams,
    config:       LlmConfig,
    mtmd_context: Option<MtmdContext>,
    audio_embed_dim: usize,
}

// SAFETY: LlamaContext<'static> no implementa Send/Sync porque la crate
// no sabe que el lifetime es válido. Aquí garantizamos que ctx y model
// siempre viven juntos en LlmModel y nunca se separan.
unsafe impl Send for LlmModel {}
unsafe impl Sync for LlmModel {}

impl LlmModel {
    pub fn new(
        model_path:  &Path,
        mmproj_path: Option<&Path>,
        config:      LlmConfig,
    ) -> Result<Self, AppError> {
        let backend = get_backend();

        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(config.n_gpu_layers);

        let model = LlamaModel::load_from_file(backend, model_path, &model_params)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))?;

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(config.context_size))
            .with_n_batch(config.batch_size)
            .with_n_ubatch(config.ubatch_size)
            .with_n_seq_max(1)
            .with_embeddings(true);

        // SAFETY: ctx referencia model. Ambos viven en este struct y ctx
        // se declaró antes que model → drop order correcto garantizado.
        let ctx_raw = model
            .new_context(backend, ctx_params.clone())
            .map_err(|e| LlmError::ContextInit(e.to_string()))?;
        let ctx: LlamaContext<'static> = unsafe { std::mem::transmute(ctx_raw) };

        let mtmd_context = if let Some(mmproj) = mmproj_path {
            let params = llama_cpp_2::mtmd::MtmdContextParams::default();
            let mtmd = MtmdContext::init_from_file(
                mmproj.to_string_lossy().as_ref(),
                &model,
                &params,
            ).map_err(|e| LlmError::MtmdInit(e.to_string()))?;
            Some(mtmd)
        } else {
            None
        };

        Ok(Self {
            ctx: Some(ctx),
            model,
            ctx_params,
            config,
            mtmd_context,
            audio_embed_dim: 2304,
        })
    }

    /// Recrea el contexto (resetea KV cache para nueva conversación).
    pub fn reset_context(&mut self) -> Result<(), AppError> {
        // Dropear ctx viejo ANTES de crear el nuevo — crítico para safety
        self.ctx = None;

        let ctx_raw = self.model
            .new_context(get_backend(), self.ctx_params.clone())
            .map_err(|e| LlmError::ContextInit(e.to_string()))?;

        // SAFETY: misma garantía que en new()
        self.ctx = Some(unsafe { std::mem::transmute(ctx_raw) });
        Ok(())
    }

    /// Borrow mutable del contexto con lifetime correcto.
    pub fn ctx_mut(&mut self) -> Result<&mut LlamaContext<'_>, AppError> {
        // SAFETY: transmute 'static → lifetime del borrow de self.
        // Seguro porque ctx no puede outlive self.
        self.ctx
            .as_mut()
            .map(|c| unsafe {
                let c: &mut LlamaContext<'static> = c;
                std::mem::transmute::<&mut LlamaContext<'static>, &mut LlamaContext<'_>>(c)
            })
            .ok_or_else(|| LlmError::NotInitialized.into())
    }

    /// Retorna (&mut ctx, &mtmd) simultáneamente.
    ///
    /// El borrow checker no puede inferir que ctx y mtmd_context son campos
    /// distintos cuando los pedimos a través de métodos `&mut self`. Este
    /// método hace el split explícitamente usando punteros raw, lo cual es
    /// seguro porque los campos no se solapan en memoria.
    pub fn split_ctx_mtmd(
        &mut self,
    ) -> Result<(&mut LlamaContext<'_>, &MtmdContext), AppError> {
        let ctx = self.ctx
            .as_mut()
            .ok_or(LlmError::NotInitialized)?;

        let mtmd = self.mtmd_context
            .as_ref()
            .ok_or(LlmError::MtmdInit("No mmproj loaded".into()))?;

        // SAFETY: ctx y mtmd_context son campos distintos del struct.
        // Creamos dos referencias que no se solapan. El raw pointer
        // evita que el borrow checker piense que ambos vienen del mismo
        // `&mut self`, cuando en realidad son regiones de memoria distintas.
        let ctx_ptr: *mut LlamaContext<'static> = ctx as *mut _;
        let ctx_ref: &mut LlamaContext<'_> = unsafe {
            std::mem::transmute(&mut *ctx_ptr)
        };

        Ok((ctx_ref, mtmd))
    }

    /// Tamaño del contexto en tokens.
    pub fn n_ctx(&self) -> u32 {
        self.ctx_params
            .n_ctx()
            .map(|n| n.get())
            .unwrap_or(4096)
    }

    /// Verifica si agregar `needed` tokens llenaría el contexto.
    pub fn context_is_full(&self, n_past: i32, needed: i32) -> bool {
        n_past + needed > self.n_ctx() as i32
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn model(&self) -> &LlamaModel            { &self.model }
    pub fn config(&self) -> &LlmConfig            { &self.config }
    pub fn audio_embed_dim(&self) -> usize         { self.audio_embed_dim }
    pub fn ctx_params(&self) -> &LlamaContextParams { &self.ctx_params }

    pub fn mtmd_context(&self) -> Option<&MtmdContext>         { self.mtmd_context.as_ref() }
    pub fn mtmd_context_mut(&mut self) -> Option<&mut MtmdContext> { self.mtmd_context.as_mut() }

    pub fn models_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }

    pub fn context_usage(&self, n_past: i32) -> (u32, u32) {
        let total = self.n_ctx();
        let used = (n_past.max(0) as u32).min(total);
        (used, total)
    }
}