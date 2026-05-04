/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Motor TTS: inferencia ONNX para síntesis de voz

use crate::{
    config::TtsConfig,
    errors::{AppError, TtsError},
    tts::utils::{load_voice_style, UnicodeProcessor},
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ort::{inputs, session::Session, value::Tensor};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};
use std::path::PathBuf;
use tauri::AppHandle;

trait OrtResultExt<T> {
    fn tts_tensor(self) -> Result<T, AppError>;
    fn tts_infer(self) -> Result<T, AppError>;
}

impl<T> OrtResultExt<T> for Result<T, ort::Error> {
    fn tts_tensor(self) -> Result<T, AppError> {
        self.map_err(|e| TtsError::Tensor(e.to_string()).into())
    }
    fn tts_infer(self) -> Result<T, AppError> {
        self.map_err(|e| TtsError::Inference(e.to_string()).into())
    }
}

pub struct TtsEngine {
    pub duration_predictor: Session,
    pub text_encoder: Session,
    pub vector_estimator: Session,
    pub vocoder: Session,
    pub unicode_processor: UnicodeProcessor,
    pub assets_dir: PathBuf,
    pub config: TtsConfig,
    pub app_handle: AppHandle,
}

impl TtsEngine {
    pub fn new(assets_dir: PathBuf, app_handle: AppHandle) -> Result<Self, AppError> {
        let onnx_dir = assets_dir.join("onnx");

        // Cargar configuración
        let config: TtsConfig = serde_json::from_str(
            &std::fs::read_to_string(onnx_dir.join("tts.json"))
                .map_err(|_e| TtsError::ConfigMissing)?,
        )
        .map_err(|e| TtsError::SessionLoad(e.to_string()))?;

        // Cargar modelos ONNX
        let load_session = |name: &str| -> Result<Session, AppError> {
            Ok(Session::builder()
                .map_err(|e| TtsError::SessionLoad(e.to_string()))?
                .commit_from_file(onnx_dir.join(format!("{}.onnx", name)))
                .map_err(|e| TtsError::SessionLoad(e.to_string()))?)
        };

        Ok(Self {
            duration_predictor: load_session("duration_predictor")?,
            text_encoder: load_session("text_encoder")?,
            vector_estimator: load_session("vector_estimator")?,
            vocoder: load_session("vocoder")?,
            unicode_processor: UnicodeProcessor::new(&assets_dir)?,
            config,
            assets_dir,
            app_handle,
        })
    }

    pub fn speak_and_play(
        &mut self,
        text: &str,
        voice_id: &str,
        lang: &str,
        speed: f32,
        denoising_steps: usize,
    ) -> Result<(), AppError> {
        let wav_bytes = self.speak(text, voice_id, lang, speed, denoising_steps)?;
        let wav_u8: Vec<u8> = wav_bytes.iter().map(|&b| b as u8).collect();

        // Parse WAV header to get sample rate + samples
        let mut cursor = std::io::Cursor::new(&wav_u8);
        let (header, samples) = parse_wav(&mut cursor)?;

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| TtsError::AudioGeneration("No output device".into()))?;

        let config = cpal::StreamConfig {
            channels: header.channels,
            sample_rate: header.sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = std::sync::Arc::new(samples);
        let samples_clone = samples.clone();
        let pos = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pos_clone = pos.clone();
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let done_clone = done.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let p = pos_clone.load(std::sync::atomic::Ordering::Relaxed);
                    for (i, sample) in data.iter_mut().enumerate() {
                        if p + i < samples_clone.len() {
                            *sample = samples_clone[p + i];
                        } else {
                            *sample = 0.0;
                            done_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    pos_clone.fetch_add(data.len(), std::sync::atomic::Ordering::Relaxed);
                },
                |err| eprintln!("TTS playback error: {err}"),
                None,
            )
            .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;

        stream
            .play()
            .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;

        // Block until done
        while !done.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(())
    }

    /// Sintetizar texto a audio
    pub fn speak(
        &mut self,
        text: &str,
        voice_id: &str,
        lang: &str,
        speed: f32,
        denoising_steps: usize,
    ) -> Result<Vec<i8>, AppError> {
        // 1. Tokenizar
        let text_ids = self.unicode_processor.encode(text, lang);
        if text_ids.is_empty() {
            return Err(TtsError::EmptyText.into());
        }
        let token_len = text_ids.len();
        let text_mask: Vec<f32> = vec![1.0; token_len];

        // 2. Cargar estilo de voz
        let voice = load_voice_style(&self.assets_dir, voice_id)?;
        let (ttl_flat, ttl_dims) = voice.style_ttl.flatten();
        let (dp_flat, dp_dims) = voice.style_dp.flatten();

        // 3. Duration predictor — bloque para dropear SessionOutputs antes de usar self de nuevo
        let mut duration: Vec<f32> = {
            let dp_out = self.duration_predictor.run(inputs![
            "text_ids"  => Tensor::from_array(([1, token_len], text_ids.clone())).tts_tensor()?,
            "style_dp"  => Tensor::from_array((dp_dims.clone(), dp_flat)).tts_tensor()?,
            "text_mask" => Tensor::from_array(([1, 1, token_len], text_mask.clone())).tts_tensor()?,
        ]).tts_infer()?;

            dp_out["duration"]
                .try_extract_array::<f32>()
                .tts_infer()?
                .iter()
                .copied()
                .collect()
        }; // dp_out y borrow de self.duration_predictor mueren aquí

        for d in duration.iter_mut() {
            *d /= speed;
        }

        // 4. Text encoder
        let (emb_shape, emb_flat): (Vec<usize>, Vec<f32>) = {
            let enc_out = self.text_encoder.run(inputs![
            "text_ids"  => Tensor::from_array(([1, token_len], text_ids)).tts_tensor()?,
            "style_ttl" => Tensor::from_array((ttl_dims.clone(), ttl_flat.clone())).tts_tensor()?,
            "text_mask" => Tensor::from_array(([1, 1, token_len], text_mask.clone())).tts_tensor()?,
        ]).tts_infer()?;

            let emb_arr = enc_out["text_emb"].try_extract_array::<f32>().tts_infer()?;
            (emb_arr.shape().to_vec(), emb_arr.iter().copied().collect())
        }; // enc_out y borrow de self.text_encoder mueren aquí

        // 5. Calcular dimensiones del latente
        let chunk_size = self.config.chunk_size_compressed();
        let latent_dim = self.config.latent_dim_compressed();
        let wav_len_max = duration.iter().sum::<f32>() * self.config.ae.sample_rate as f32;
        let latent_len = ((wav_len_max as usize) + chunk_size - 1) / chunk_size;
        let latent_total = latent_dim * latent_len;

        // 6. Generar ruido gaussiano y máscara
        let normal =
            Normal::new(0.0f32, 1.0f32).map_err(|e| TtsError::AudioGeneration(e.to_string()))?;
        let mut rng = ThreadRng::default();

        let mut noisy_latent = vec![0.0f32; latent_total];
        let mut latent_mask = vec![0.0f32; latent_len];

        for d in 0..latent_dim {
            for t in 0..latent_len {
                noisy_latent[d * latent_len + t] = normal.sample(&mut rng);
            }
        }
        for t in 0..latent_len {
            latent_mask[t] = 1.0;
        }

        // 7. Pre-construir tensores reutilizables (no dependen de self)
        let text_emb_tensor = Tensor::from_array((emb_shape, emb_flat)).tts_tensor()?;
        let style_ttl_tensor = Tensor::from_array((ttl_dims, ttl_flat)).tts_tensor()?;
        let text_mask_tensor = Tensor::from_array(([1, 1, token_len], text_mask)).tts_tensor()?;
        let mut current_step_tensor = Tensor::from_array(([1], vec![0.0f32])).tts_tensor()?;
        let total_step_tensor =
            Tensor::from_array(([1], vec![denoising_steps as f32])).tts_tensor()?;
        let lat_shape = vec![1, latent_dim, latent_len];
        let mask_shape = vec![1, 1, latent_len];

        // 8. Denoising loop
        for step in 0..denoising_steps {
            // Mutar tensor de step en bloque propio para soltar borrow antes de inputs![]
            {
                let mut arr = current_step_tensor
                    .try_extract_array_mut::<f32>()
                    .tts_infer()?;
                arr[0] = step as f32;
            }

            // Extraer a Vec owned para que SessionOutputs se dropee antes del siguiente run
            let new_noisy: Vec<f32> = {
                let out = self.vector_estimator.run(inputs![
                "noisy_latent" => Tensor::from_array((lat_shape.clone(), noisy_latent.clone())).tts_tensor()?,
                "text_emb"     => &text_emb_tensor,
                "style_ttl"    => &style_ttl_tensor,
                "latent_mask"  => Tensor::from_array((mask_shape.clone(), latent_mask.clone())).tts_tensor()?,
                "text_mask"    => &text_mask_tensor,
                "current_step" => &current_step_tensor,
                "total_step"   => &total_step_tensor,
            ]).tts_infer()?;

                out["denoised_latent"]
                    .try_extract_array::<f32>()
                    .tts_infer()?
                    .iter()
                    .copied()
                    .collect()
            }; // out y borrow de self.vector_estimator mueren aquí

            noisy_latent.copy_from_slice(&new_noisy);
        }

        // 9. Vocoder
        let waveform: Vec<f32> = {
            let voc_out = self
                .vocoder
                .run(inputs![
                    "latent" => Tensor::from_array((lat_shape, noisy_latent)).tts_tensor()?,
                ])
                .tts_infer()?;

            voc_out
                .get("wav_tts")
                .or_else(|| voc_out.get("waveform"))
                .or_else(|| voc_out.get("audio"))
                .ok_or(TtsError::AudioGeneration("No audio output".into()))?
                .try_extract_array::<f32>()
                .tts_infer()?
                .iter()
                .copied()
                .collect()
        }; // voc_out y borrow de self.vocoder mueren aquí

        // 10. Convertir a WAV
        self.waveform_to_wav_bytes(&waveform)
    }

    /// Convertir waveform a bytes WAV en memoria
    fn waveform_to_wav_bytes(&self, waveform: &[f32]) -> Result<Vec<i8>, AppError> {
        use hound::{SampleFormat, WavSpec, WavWriter};
        use std::io::Cursor;

        let mut buffer = Cursor::new(Vec::new());
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.config.ae.sample_rate as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        {
            let mut writer = WavWriter::new(&mut buffer, spec)
                .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;
            for s in waveform {
                writer
                    .write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;
            }
            writer
                .finalize()
                .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;
        }

        let bytes = buffer.into_inner();
        // Safe: Vec<u8> y Vec<i8> tienen misma representación
        Ok(unsafe { std::mem::transmute::<Vec<u8>, Vec<i8>>(bytes) })
    }

    /// Listar voces disponibles
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
fn parse_wav(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<(WavHeader, Vec<f32>), AppError> {
    use std::io::Read;
    let mut header = [0u8; 44];
    cursor
        .read_exact(&mut header)
        .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;

    let channels = u16::from_le_bytes([header[22], header[23]]);
    let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
    let bits_per_sample = u16::from_le_bytes([header[34], header[35]]);

    let mut raw = Vec::new();
    cursor
        .read_to_end(&mut raw)
        .map_err(|e| TtsError::AudioGeneration(e.to_string()))?;

    let samples: Vec<f32> = match bits_per_sample {
        16 => raw
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
            .collect(),
        32 => raw
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
        _ => {
            return Err(
                TtsError::AudioGeneration(format!("Unsupported bits: {bits_per_sample}")).into(),
            )
        }
    };

    Ok((
        WavHeader {
            channels,
            sample_rate,
        },
        samples,
    ))
}

struct WavHeader {
    channels: u16,
    sample_rate: u32,
}
