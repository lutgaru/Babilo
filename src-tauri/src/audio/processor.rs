/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Procesamiento de señal de audio: Mel spectrograms, FFT, etc.

use ndarray::{Array2, arr1};
use realfft::RealFftPlanner;
use std::f32::consts::PI;
use crate::config::LlmAudioConfig;

#[allow(dead_code)]
pub struct MelPreprocessor {
    fft_size: usize,
    mel_filters: Array2<f32>,
    window: Vec<f32>,
    mel_floor: f32,
    sample_rate: u32,
    n_mels: usize,
}

impl MelPreprocessor {
    pub fn new(sample_rate: u32, n_mels: usize, n_fft: usize) -> Self {
        // Ventana de Hann
        let window: Vec<f32> = (0..n_fft)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / n_fft as f32).cos()))
            .collect();
        
        // TODO: Implementar banco de filtros Mel real
        // Por ahora usamos un placeholder
        let mel_filters = Array2::zeros((n_mels, n_fft / 2 + 1));
        
        Self {
            fft_size: n_fft,
            mel_filters,
            window,
            mel_floor: 1e-3,
            sample_rate,
            n_mels,
        }
    }

    /// Procesa audio crudo → features para el modelo
    pub fn process(&self, audio: &[f32], config: &LlmAudioConfig) -> Vec<Array2<f32>> {
        let chunk_samples = config.samples_per_chunk();
        let mut chunks = Vec::new();
        
        for chunk in audio.chunks(chunk_samples) {
            if chunk.len() < config.window_size { 
                continue; 
            }
            let mel_spec = self.audio_to_mel(chunk, config);
            let mel_log = mel_spec.mapv(|x| (x.max(self.mel_floor)).ln());
            chunks.push(mel_log);
        }
        chunks
    }

    /// Convierte un chunk de audio a espectrograma Mel
    fn audio_to_mel(&self, audio: &[f32], config: &LlmAudioConfig) -> Array2<f32> {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(config.window_size);
        
        let n_frames = (audio.len() - config.window_size) / config.hop_size + 1;
        let mut mel_spec = Array2::zeros((n_frames, config.mel_bins));
        
        for frame_idx in 0..n_frames {
            let start = frame_idx * config.hop_size;
            let end = start + config.window_size;
            
            // Aplicar ventana
            let mut frame: Vec<f32> = audio[start..end]
                .iter()
                .zip(&self.window)
                .map(|(&s, &w)| s * w)
                .collect();
            
            // FFT
            let mut spectrum = fft.make_output_vec();
            if fft.process(&mut frame, &mut spectrum).is_err() {
                continue;
            }
            
            // Magnitudes
            let magnitudes: Vec<f32> = spectrum.iter()
                .take(config.window_size / 2 + 1)
                .map(|c| c.norm())
                .collect();
            
            // Proyección Mel (placeholder)
            let mel_vec = self.mel_filters.dot(&arr1(&magnitudes));
            mel_spec.row_mut(frame_idx).assign(&mel_vec);
        }
        mel_spec
    }

    /// Resampling simple por interpolación lineal
    pub fn resample(audio: &[f32], src_rate: f32, target_rate: f32) -> Vec<f32> {
        if (src_rate - target_rate).abs() < f32::EPSILON {
            return audio.to_vec();
        }

        let ratio = src_rate / target_rate;
        let mut resampled = Vec::with_capacity((audio.len() as f32 / ratio) as usize);
        
        let mut i = 0.0;
        while i < audio.len() as f32 {
            let idx = i as usize;
            let next_idx = (idx + 1).min(audio.len() - 1);
            let frac = i - idx as f32;
            
            let value = audio[idx] * (1.0 - frac) + audio[next_idx] * frac;
            resampled.push(value);
            
            i += ratio;
        }
        resampled
    }
}