use ort::{inputs, session::Session, value::Tensor};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};
use serde::Deserialize;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager; // ← Agregar este import
                    // ── Config desde tts.json ─────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct TtsConfig {
    pub ae: AeConfig,
    pub ttl: TtlConfig,
}

#[derive(Deserialize, Debug)]
pub struct AeConfig {
    pub sample_rate: i32,
    pub base_chunk_size: i32,
}

#[derive(Deserialize, Debug)]
pub struct TtlConfig {
    pub chunk_compress_factor: i32,
    pub latent_dim: i32,
}

// ── Voice style JSON ──────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
pub struct VoiceStyle {
    pub style_ttl: StyleTensor,
    pub style_dp: StyleTensor,
}

#[derive(Deserialize, Debug)]
pub struct StyleTensor {
    pub data: Vec<Vec<Vec<f32>>>,
    pub dims: Vec<usize>,
}

impl StyleTensor {
    pub fn flatten(&self) -> (Vec<f32>, Vec<usize>) {
        let flat = self
            .data
            .iter()
            .flat_map(|d2| d2.iter().flat_map(|d3| d3.iter().copied()))
            .collect();
        (flat, self.dims.clone())
    }
}

// ── Unicode Processor ─────────────────────────────────────────────────────

pub struct UnicodeProcessor {
    indexer: Vec<i64>,
}

impl UnicodeProcessor {
    pub fn new(assets_dir: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let path = assets_dir.join("onnx").join("unicode_indexer.json");
        let json = std::fs::read_to_string(&path)?;
        let indexer: Vec<i64> = serde_json::from_str(&json)?;
        Ok(Self { indexer })
    }

    pub fn encode(&self, text: &str, lang: &str) -> Vec<i64> {
        let wrapped = format!("<{}>{}</{}>", lang, text, lang);
        wrapped
            .chars()
            .map(|c| {
                let idx = c as usize;
                if idx < self.indexer.len() {
                    self.indexer[idx]
                } else {
                    -1
                }
            })
            .collect()
    }
}

// ── Motor TTS ─────────────────────────────────────────────────────────────

pub struct TtsEngine {
    pub duration_predictor: Session,
    pub text_encoder: Session,
    pub vector_estimator: Session,
    pub vocoder: Session,
    pub unicode_processor: UnicodeProcessor,
    pub assets_dir: PathBuf,
    pub sample_rate: i32,
    pub base_chunk_size: i32,
    pub chunk_compress: i32,
    pub latent_dim: i32,
    pub app_handle: AppHandle, // ← Nuevo campo
}

impl TtsEngine {
    pub fn new(
        assets_dir: PathBuf,
        app_handle: AppHandle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let onnx_dir = assets_dir.join("onnx");

        // Leer config primero — los dims vienen de aquí
        let cfg: TtsConfig =
            serde_json::from_str(&std::fs::read_to_string(onnx_dir.join("tts.json"))?)?;

        let duration_predictor =
            Session::builder()?.commit_from_file(onnx_dir.join("duration_predictor.onnx"))?;
        let text_encoder =
            Session::builder()?.commit_from_file(onnx_dir.join("text_encoder.onnx"))?;
        let vector_estimator =
            Session::builder()?.commit_from_file(onnx_dir.join("vector_estimator.onnx"))?;
        let vocoder = Session::builder()?.commit_from_file(onnx_dir.join("vocoder.onnx"))?;
        let unicode_processor = UnicodeProcessor::new(&assets_dir)?;

        Ok(Self {
            duration_predictor,
            text_encoder,
            vector_estimator,
            vocoder,
            unicode_processor,
            sample_rate: cfg.ae.sample_rate,
            base_chunk_size: cfg.ae.base_chunk_size,
            chunk_compress: cfg.ttl.chunk_compress_factor,
            latent_dim: cfg.ttl.latent_dim,
            assets_dir,
            app_handle, // ← Inicializar el nuevo campo
        })
    }

    fn load_voice(&self, voice_id: &str) -> Result<VoiceStyle, Box<dyn std::error::Error>> {
        let path = self
            .assets_dir
            .join("voice_styles")
            .join(format!("{}.json", voice_id));
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn speak(
        &mut self,
        text: &str,
        voice_id: &str,
        lang: &str,
        speed: f32,
        total_steps: usize,
    ) -> Result<Vec<i8>, Box<dyn std::error::Error>> {
        // ── 1. Tokenizar ──────────────────────────────────────────────────
        let text_ids = self.unicode_processor.encode(text, lang);
        let token_len = text_ids.len();
        if token_len == 0 {
            return Err("Texto vacío".into());
        }

        // text_mask: [1, 1, token_len]
        let text_mask: Vec<f32> = vec![1.0; token_len];

        // ── 2. Voice style ────────────────────────────────────────────────
        let voice = self.load_voice(voice_id)?;
        let (ttl_flat, ttl_dims) = voice.style_ttl.flatten(); // e.g. [1, 50, 256]
        let (dp_flat, dp_dims) = voice.style_dp.flatten(); // e.g. [1, 8,  16]

        // ── 3. Duration predictor ─────────────────────────────────────────
        let dp_out = self.duration_predictor.run(inputs![
            "text_ids"  => Tensor::from_array(([1usize, token_len], text_ids.clone()))?,
            "style_dp"  => Tensor::from_array((dp_dims.clone(), dp_flat))?,
            "text_mask" => Tensor::from_array(([1usize, 1usize, token_len], text_mask.clone()))?,
        ])?;

        let dur_arr = dp_out["duration"].try_extract_array::<f32>()?;
        let mut duration: Vec<f32> = dur_arr.iter().copied().collect();
        for d in duration.iter_mut() {
            *d /= speed;
        }

        // ── 4. Text encoder ───────────────────────────────────────────────
        let enc_out = self.text_encoder.run(inputs![
            "text_ids"   => Tensor::from_array(([1usize, token_len], text_ids))?,
            "style_ttl"  => Tensor::from_array((ttl_dims.clone(), ttl_flat.clone()))?,
            "text_mask"  => Tensor::from_array(([1usize, 1usize, token_len], text_mask.clone()))?,
        ])?;

        let emb_arr = enc_out["text_emb"].try_extract_array::<f32>()?;
        let emb_shape: Vec<usize> = emb_arr.shape().to_vec();
        let emb_flat: Vec<f32> = emb_arr.iter().copied().collect();

        // ── 5. Calcular dimensiones del latente (igual que helper.rs) ─────
        //
        //  chunk_size  = base_chunk_size * chunk_compress_factor
        //  latent_dim  = latent_dim      * chunk_compress_factor   ← este es el 144
        //  wav_len_max = sum(duration) * sample_rate
        //  latent_len  = ceil(wav_len_max / chunk_size)
        //
        let chunk_size = (self.base_chunk_size * self.chunk_compress) as usize;
        let latent_dim = (self.latent_dim * self.chunk_compress) as usize; // ← 144
        let wav_len_max = duration.iter().sum::<f32>() * self.sample_rate as f32;
        let latent_len = ((wav_len_max as usize) + chunk_size - 1) / chunk_size;

        // ── 6. Noisy latent + mask (Gaussiano, igual que helper.rs) ───────
        let latent_total = latent_dim * latent_len;
        let normal = Normal::new(0.0f32, 1.0f32)?;
        let mut rng = ThreadRng::default();

        // Calcular latent_lengths por token para la máscara
        let latent_lengths: Vec<usize> = {
            let wav_lengths: Vec<usize> = duration
                .iter()
                .map(|&d| (d * self.sample_rate as f32) as usize)
                .collect();
            wav_lengths
                .iter()
                .map(|&l| (l + chunk_size - 1) / chunk_size)
                .collect()
        };

        let mut noisy_latent = vec![0.0f32; latent_total];
        let mut latent_mask = vec![0.0f32; latent_len]; // [1, 1, latent_len]

        // Llenar ruido y máscara
        let active_frames = *latent_lengths.iter().max().unwrap_or(&latent_len);
        for d in 0..latent_dim {
            for t in 0..latent_len {
                let noise = normal.sample(&mut rng);
                let is_active = t < active_frames;
                noisy_latent[d * latent_len + t] = if is_active { noise } else { 0.0 };
            }
        }
        for t in 0..active_frames.min(latent_len) {
            latent_mask[t] = 1.0;
        }

        // shapes para el modelo
        let lat_shape = vec![1usize, latent_dim, latent_len];
        let mask_shape = vec![1usize, 1usize, latent_len];

        // ── 7. Denoising loop OPTIMIZADO ─────────────────────────────
        // Pre-convertir tensors que NO cambian (fuera del loop)
        let text_emb_tensor = Tensor::from_array((emb_shape.clone(), emb_flat.clone()))?;
        let style_ttl_tensor = Tensor::from_array((ttl_dims.clone(), ttl_flat.clone()))?;
        let text_mask_tensor =
            Tensor::from_array(([1usize, 1usize, token_len], text_mask.clone()))?;
        let mut current_step_tensor = Tensor::from_array(([1usize], vec![0.0f32]))?; // Reutilizar buffer
        let total_step_tensor = Tensor::from_array(([1usize], vec![total_steps as f32]))?;

        // Buffer reutilizable para noisy_latent (evitar realloc)
        let mut latent_buffer = vec![0.0f32; latent_total];

        for step in 0..total_steps {
            // Actualizar solo el valor del step actual
            current_step_tensor.try_extract_array_mut::<f32>()?[0] = step as f32;

            // Crear tensor de noisy_latent SIN clone del Vec (usar referencia)
            let out = self.vector_estimator.run(inputs![
                "noisy_latent" => Tensor::from_array((lat_shape.clone(), noisy_latent.clone()))?,  // ← Este sí necesita clone por mutabilidad
                "text_emb"     => &text_emb_tensor,    // ✅ Referencia, no clone
                "style_ttl"    => &style_ttl_tensor,   // ✅ Referencia
                "latent_mask"  => Tensor::from_array((mask_shape.clone(), latent_mask.clone()))?,
                "text_mask"    => &text_mask_tensor,   // ✅ Referencia
                "current_step" => &current_step_tensor,// ✅ Referencia
                "total_step"   => &total_step_tensor,  // ✅ Referencia
            ])?;

            // Extraer directamente al buffer reutilizable
            let pred = out["denoised_latent"].try_extract_array::<f32>()?;
            noisy_latent.copy_from_slice(pred.as_slice().unwrap()); // ✅ memcpy en lugar de iter+collect
        }

        // ── 8. Vocoder ────────────────────────────────────────────────────
        let voc_out = self.vocoder.run(inputs![
            "latent" => Tensor::from_array((lat_shape, noisy_latent))?,
        ])?;

        let wav_arr = voc_out
            .get("wav_tts")
            .or_else(|| voc_out.get("waveform"))
            .or_else(|| voc_out.get("audio"))
            .ok_or("vocoder: output no encontrado")?
            .try_extract_array::<f32>()?;

        let waveform: Vec<f32> = wav_arr.iter().copied().collect();

        // ── 9. Generar WAV en memoria (sin tocar disco) ───────────────────────
        use hound::{SampleFormat, WavSpec, WavWriter};
        use std::io::Cursor;

        let mut buffer = Cursor::new(Vec::new());
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        {
            let mut writer = WavWriter::new(&mut buffer, spec)?;
            for s in waveform {
                writer.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
            }
            writer.finalize()?;
        }

        // 🔑 Convertir Vec<u8> → Vec<i8> (misma representación en memoria)
        let bytes = buffer.into_inner();
        Ok(unsafe { std::mem::transmute::<Vec<u8>, Vec<i8>>(bytes) })
    }

    pub fn list_voices(&self) -> Vec<String> {
        let dir = self.assets_dir.join("voice_styles");
        std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        name.strip_suffix(".json").map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

pub fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Raíz del proyecto no encontrada")
        .join("assets")
}
