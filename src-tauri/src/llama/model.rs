//! Gestión del modelo llama.cpp: carga, contexto, configuración

use llama_cpp_2::{
    context::params::LlamaContextParams,
    context::LlamaContext,
    llama_backend::LlamaBackend,
    model::{params::LlamaModelParams, LlamaModel},
    mtmd::MtmdContext,
};
use std::path::{Path, PathBuf};
use std::pin::Pin;
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

/// Innards del modelo en el heap, pineados para que nunca se muevan.
///
/// # Por qué Pin<Box<...>>
///
/// `LlamaContext<'a>` guarda un puntero interno al `LlamaModel` del que
/// fue creado. Si el struct que contiene ambos se mueve en memoria (stack
/// o realloc), el puntero interno queda colgando → ACCESS_VIOLATION en
/// release (el compilador optimiza más agresivamente).
///
/// `Pin<Box<Inner>>` garantiza que la dirección de `Inner` en el heap
/// nunca cambia después de la construcción, haciendo el invariante seguro
/// incluso con optimizaciones de release.
struct Inner {
    // ctx ANTES de model → se dropea primero (drop order inverso al de declaración)
    ctx:          Option<LlamaContext<'static>>,
    model:        LlamaModel,
    mtmd_context: Option<MtmdContext>,
}

pub struct LlmModel {
    inner:      Pin<Box<Inner>>,
    ctx_params: LlamaContextParams,
    config:     LlmConfig,
    audio_embed_dim: usize,
}

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

        // Construir Inner en el heap ANTES de crear el contexto,
        // para que la dirección de `model` sea definitiva cuando
        // new_context() guarda el puntero interno.
        let mut inner = Box::new(Inner {
            ctx:   None,
            model,
            mtmd_context,
        });

        // SAFETY: inner.model ya está en su dirección final (heap).
        // Pin garantiza que nunca se moverá. ctx se declara antes que
        // model en Inner → drop order correcto.
        let ctx_raw = inner.model
            .new_context(backend, ctx_params.clone())
            .map_err(|e| LlmError::ContextInit(e.to_string()))?;
        inner.ctx = Some(unsafe { std::mem::transmute(ctx_raw) });

        Ok(Self {
            inner: Pin::new(inner),
            ctx_params,
            config,
            audio_embed_dim: 2304,
        })
    }

    /// Recrea el contexto (resetea KV cache para nueva conversación).
    pub fn reset_context(&mut self) -> Result<(), AppError> {
        // SAFETY: no movemos Inner, solo mutamos su contenido.
        let inner = unsafe { self.inner.as_mut().get_unchecked_mut() };

        // Dropear ctx viejo ANTES de crear el nuevo
        inner.ctx = None;

        let ctx_raw = inner.model
            .new_context(get_backend(), self.ctx_params.clone())
            .map_err(|e| LlmError::ContextInit(e.to_string()))?;

        inner.ctx = Some(unsafe { std::mem::transmute(ctx_raw) });
        Ok(())
    }

    /// Borrow mutable del contexto con lifetime correcto.
    pub fn ctx_mut(&mut self) -> Result<&mut LlamaContext<'_>, AppError> {
        // SAFETY: no movemos Inner, solo tomamos referencia mutable a ctx.
        let inner = unsafe { self.inner.as_mut().get_unchecked_mut() };
        inner.ctx
            .as_mut()
            .map(|c| unsafe {
                std::mem::transmute::<&mut LlamaContext<'static>, &mut LlamaContext<'_>>(c)
            })
            .ok_or_else(|| LlmError::NotInitialized.into())
    }

    /// Retorna (&mut ctx, &mtmd) simultáneamente via borrow split explícito.
    pub fn split_ctx_mtmd(
        &mut self,
    ) -> Result<(&mut LlamaContext<'_>, &MtmdContext), AppError> {
        // SAFETY: no movemos Inner.
        let inner = unsafe { self.inner.as_mut().get_unchecked_mut() };

        let ctx = inner.ctx
            .as_mut()
            .ok_or(LlmError::NotInitialized)?;

        let mtmd = inner.mtmd_context
            .as_ref()
            .ok_or(LlmError::MtmdInit("No mmproj loaded".into()))?;

        // SAFETY: ctx y mtmd_context son campos distintos de Inner → no se solapan.
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

    pub fn model(&self) -> &LlamaModel {
        &self.inner.model
    }

    pub fn config(&self) -> &LlmConfig            { &self.config }
    pub fn audio_embed_dim(&self) -> usize         { self.audio_embed_dim }
    pub fn ctx_params(&self) -> &LlamaContextParams { &self.ctx_params }

    pub fn mtmd_context(&self) -> Option<&MtmdContext> {
        self.inner.mtmd_context.as_ref()
    }

    pub fn mtmd_context_mut(&mut self) -> Option<&mut MtmdContext> {
        // SAFETY: no movemos Inner.
        let inner = unsafe { self.inner.as_mut().get_unchecked_mut() };
        inner.mtmd_context.as_mut()
    }

    pub fn context_usage(&self, n_past: i32) -> (u32, u32) {
        let total = self.n_ctx();
        let used  = (n_past.max(0) as u32).min(total);
        (used, total)
    }

    pub fn models_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Raíz del proyecto no encontrada")
            .join("models")
    }
}