use crate::cpu::{AudioBufferConsumer, AudioConfig};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use ringbuf::traits::Consumer;

pub struct AudioHandler {
    _stream: Stream,
}

impl AudioHandler {
    pub fn get_audio_config() -> AudioConfig {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device found");
        let config = device.default_output_config().unwrap();

        AudioConfig {
            sample_rate: config.sample_rate().0,
            channels: config.channels() as usize,
            ..Default::default()
        }
    }

    pub fn init(consumer: AudioBufferConsumer) -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device found");
        let config = device.default_output_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::init_stream::<f32>(&device, &config.into(), consumer),
            cpal::SampleFormat::I16 => Self::init_stream::<i16>(&device, &config.into(), consumer),
            cpal::SampleFormat::U16 => Self::init_stream::<u16>(&device, &config.into(), consumer),
            _ => panic!("Unsupported sample format!"),
        };

        Self { _stream: stream }
    }

    fn init_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut consumer: AudioBufferConsumer,
    ) -> Stream
    where
        T: SizedSample + FromSample<f32>,
    {
        let err_fn =
            |err| web_sys::console::error_1(&format!("Audio stream error: {}", err).into());

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _| {
                    for sample in data.iter_mut() {
                        *sample = match consumer.try_pop() {
                            Some(s) => T::from_sample::<f32>(s),
                            None => T::from_sample::<f32>(0.0),
                        };
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        stream
    }
}
