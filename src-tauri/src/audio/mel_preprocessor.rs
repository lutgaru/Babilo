use ndarray::{Array1, Array2, arr1};  // ✅ arr1 en vez de array1
use realfft::RealFftPlanner;
use std::f32::consts::PI;
use crate::audio::capture::AudioConfig;  // ✅ Importar AudioConfig

pub struct MelPreprocessor {
    fft_size: usize,
    mel_filters: Array2<f32>,
    window: Vec<f32>,
    mel_floor: f32,
}

impl MelPreprocessor {
    pub fn new(sample_rate: u32, n_mels: usize, n_fft: usize) -> Self {
        let window: Vec<f32> = (0..n_fft)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / n_fft as f32).cos()))
            .collect();
        
        // 🔧 Placeholder: en producción usar crate 'mel-filter-bank'
        let mel_filters = Array2::zeros((n_mels, n_fft / 2 + 1));
        
        Self {
            fft_size: n_fft,
            mel_filters,
            window,
            mel_floor: 1e-3,
        }
    }

    /// Procesa audio crudo → features para Gemma 4
    pub fn process(&self, audio: &[f32], config: &AudioConfig) -> Vec<Array2<f32>> {
        let chunk_samples = config.chunk_duration_secs as usize * config.sample_rate as usize;
        let mut chunks = Vec::new();
        
        for chunk in audio.chunks(chunk_samples) {
            if chunk.len() < config.window_size { continue; }
            let mel_spec = self.audio_to_mel(chunk, config);
            let mel_log = mel_spec.mapv(|x| (x.max(self.mel_floor)).ln());
            chunks.push(mel_log);
        }
        chunks
    }

    fn audio_to_mel(&self, audio: &[f32], config: &AudioConfig) -> Array2<f32> {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(config.window_size);
        
        let n_frames = (audio.len() - config.window_size) / config.hop_size + 1;
        let mut mel_spec = Array2::zeros((n_frames, config.mel_bins));
        
        for frame_idx in 0..n_frames {
            let start = frame_idx * config.hop_size;
            let end = start + config.window_size;
            
            let mut frame: Vec<f32> = audio[start..end]
                .iter()
                .zip(&self.window)
                .map(|(&s, &w)| s * w)
                .collect();
            
            let mut spectrum = fft.make_output_vec();
            fft.process(&mut frame, &mut spectrum).unwrap();
            
            let magnitudes: Vec<f32> = spectrum.iter()
                .take(config.window_size / 2 + 1)
                .map(|c| c.norm())
                .collect();
            
            // ✅ Usar arr1 en vez de array1
            let mel_vec = self.mel_filters.dot(&arr1(&magnitudes));
            mel_spec.row_mut(frame_idx).assign(&mel_vec);
        }
        mel_spec
    }
}