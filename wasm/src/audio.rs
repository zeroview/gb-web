use cpal::{
    FromSample, SizedSample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use dmg_2025_core::AudioBufferConsumer;
use ringbuf::traits::Consumer;
use std::sync::{Arc, RwLock};

pub struct AudioHandler {
    pub volume: Arc<RwLock<f32>>,
    pub paused: Arc<RwLock<bool>>,
    pub sample_rate: u32,
    pub channels: usize,
    pub sample_capacity: usize,
    sample_format: cpal::SampleFormat,
    device: cpal::Device,
    config: cpal::StreamConfig,
    stream: Option<Stream>,
}

impl AudioHandler {
    const BUFFER_CAPACITY_MS: f32 = 100.0;

    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device found");
        let config = device.default_output_config().unwrap();

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_capacity =
            (((Self::BUFFER_CAPACITY_MS / 1000.0) * sample_rate as f32) as usize) * channels;
        let sample_format = config.sample_format();

        Self {
            volume: Arc::new(RwLock::new(1.0)),
            paused: Arc::new(RwLock::new(true)),
            sample_rate,
            channels,
            sample_capacity,
            sample_format,
            device,
            config: config.into(),
            stream: None,
        }
    }

    pub fn init_playback(&mut self, consumer: AudioBufferConsumer) {
        match self.sample_format {
            cpal::SampleFormat::F32 => self.init_stream::<f32>(consumer),
            cpal::SampleFormat::I16 => self.init_stream::<i16>(consumer),
            cpal::SampleFormat::U16 => self.init_stream::<u16>(consumer),
            _ => panic!("Unsupported sample format!"),
        };
        *self.paused.write().unwrap() = false;
    }

    fn init_stream<T>(&mut self, mut consumer: AudioBufferConsumer)
    where
        T: SizedSample + FromSample<f32>,
    {
        let err_fn = |err| log::error!("Audio stream error: {}", err);

        let volume_ref = Arc::clone(&self.volume);
        let paused_ref = Arc::clone(&self.paused);
        let mut last_sample = 0.0;
        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |data: &mut [T], _| {
                    if *paused_ref.read().unwrap() {
                        data.fill(T::from_sample::<f32>(0.0));
                        return;
                    }
                    let volume = volume_ref.read().unwrap();
                    let mut late = false;
                    for sample in data.iter_mut() {
                        *sample = match consumer.try_pop() {
                            Some(s) => {
                                let new_sample = s * 0.01 * *volume;
                                last_sample = new_sample;
                                T::from_sample::<f32>(new_sample)
                            }
                            None => {
                                late = true;
                                T::from_sample::<f32>(last_sample)
                            }
                        };
                    }
                    if late {
                        log::warn!("Audio producer is late");
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        self.stream = Some(stream);
    }
}
