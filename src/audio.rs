use super::*;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample, Stream,
};
use ringbuf::{
    storage::Heap,
    traits::{Consumer, Producer, Split},
    wrap::caching::Caching,
    HeapRb, SharedRb,
};

const AUDIO_DELAY_MS: f32 = 100.0;

pub struct AudioHandler {
    pub sample_rate: u32,
    pub channels: usize,
    stream: Stream,
    producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
}

impl AudioHandler {
    pub fn init() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device found");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let sample_capacity =
            (((AUDIO_DELAY_MS / 1000.0) * sample_rate as f32) as usize) * channels;
        let ring = HeapRb::<f32>::new(sample_capacity);
        let (producer, consumer) = ring.split();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                Self::init_stream::<f32, _>(&device, &config.into(), consumer)
            }
            cpal::SampleFormat::I16 => {
                Self::init_stream::<i16, _>(&device, &config.into(), consumer)
            }
            cpal::SampleFormat::U16 => {
                Self::init_stream::<u16, _>(&device, &config.into(), consumer)
            }
            _ => panic!("Unsupported sample format!"),
        };

        Self {
            sample_rate,
            channels,
            stream,
            producer,
        }
    }

    fn init_stream<T, C>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut consumer: C,
    ) -> Stream
    where
        T: SizedSample + FromSample<f32>,
        C: Consumer<Item = f32> + Send + 'static,
    {
        let err_fn =
            |err| web_sys::console::error_1(&format!("Audio stream error: {}", err).into());

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _| {
                    let mut fell_behind = false;
                    for sample in data.iter_mut() {
                        *sample = match consumer.try_pop() {
                            Some(s) => T::from_sample::<f32>(s),
                            None => {
                                fell_behind = true;
                                T::from_sample::<f32>(0.0)
                            }
                        };
                    }
                    if fell_behind {
                        eprintln!("Input audio stream fell behind");
                    }
                },
                err_fn,
                None,
            )
            .unwrap();
        stream.play().unwrap();
        stream
    }

    pub fn update_audio(&mut self, data: &mut Vec<f32>) {
        for sample in data.iter() {
            if self.producer.try_push(*sample).is_err() {
                eprintln!("Output audio stream fell behind");
                break;
            }
        }
        data.clear();
    }
}
