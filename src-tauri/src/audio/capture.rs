use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, SampleRate, Stream, StreamConfig,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

// ✅ AudioConfig definido aquí para que todos lo usen
#[derive(Clone, Debug)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_secs: u32,
    pub mel_bins: usize,
    pub window_size: usize,
    pub hop_size: usize,
}

pub struct AudioCapture {
    device: Device,
    config: StreamConfig,
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl AudioCapture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device found")?;

        // ✅ Usar default_config y modificar solo lo necesario
        let mut config: cpal::StreamConfig = device.default_input_config()?.into();
        config.sample_rate = cpal::SampleRate(16000);
        config.channels = 1;

        Ok(Self {
            device,
            config,
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(16000 * 30))),
        })
    }

    pub fn start(&mut self) -> Result<Stream, Box<dyn std::error::Error>> {
        self.running.store(true, Ordering::SeqCst);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);

        let err_fn = |err| eprintln!("❌ Error de audio: {}", err);

        // ✅ Manejar ambos formatos de muestra correctamente
        let stream = match self.device.default_input_config()?.sample_format() {
            SampleFormat::I16 => self.device.build_input_stream(
                &self.config,
                move |data: &[i16], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        for sample in data {
                            buf.push(*sample as f32 / 32768.0);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::F32 => self.device.build_input_stream(
                &self.config,
                move |data: &[f32], _: &_| {
                    if running.load(Ordering::SeqCst) {
                        let mut buf = buffer.lock().unwrap();
                        buf.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err("Formato de muestra no soportado".into()),
        };

        stream.play()?;
        Ok(stream)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn take_buffer(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// Config para Gemma 4: 30s chunks, 16kHz, mono
    pub fn gemma4_config() -> AudioConfig {
        AudioConfig {
            sample_rate: 16000,
            channels: 1,
            chunk_duration_secs: 30,
            mel_bins: 128,
            window_size: 320, // 20ms @ 16kHz
            hop_size: 160,    // 10ms hop
        }
    }
}
