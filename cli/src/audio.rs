use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, OutputCallbackInfo, Sample, SampleRate, Stream, StreamConfig};
use std::error::Error;
use std::sync::mpsc::{self, Sender};
use tracing::error;

const MAX_SAMPLE_RATE: u32 = 48000;

struct OutputStream {
    sender: Sender<(i16, i16)>,
    _stream: Stream,
}

pub struct AudioReceiver {
    device: Device,
    sample_rate: u32,
    stream: Option<OutputStream>,
}

impl AudioReceiver {
    pub fn new(sample_rate: u32) -> Result<AudioReceiver, Box<dyn Error>> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("No audio output device available");

        let stream = Self::stream_from(&device, sample_rate)?;

        Ok(AudioReceiver {
            device,
            sample_rate,
            stream,
        })
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), Box<dyn Error>> {
        if sample_rate == self.sample_rate {
            return Ok(());
        }

        self.sample_rate = sample_rate;
        self.stream = Self::stream_from(&self.device, sample_rate)?;

        Ok(())
    }

    fn stream_from(
        device: &Device,
        sample_rate: u32,
    ) -> Result<Option<OutputStream>, Box<dyn Error>> {
        if sample_rate > MAX_SAMPLE_RATE {
            return Ok(None);
        }

        let config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Default,
        };

        let (sender, receiver) = mpsc::channel::<(i16, i16)>();

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [i16], _: &OutputCallbackInfo| {
                let mut prev_sample = (Sample::EQUILIBRIUM, Sample::EQUILIBRIUM);

                for sample_out in output.chunks_exact_mut(2) {
                    if let Ok(sample_in) = receiver.try_recv() {
                        sample_out[0] = sample_in.0;
                        sample_out[1] = sample_in.1;
                        prev_sample = sample_in;
                    } else {
                        sample_out[0] = prev_sample.0;
                        sample_out[1] = prev_sample.1;
                    }
                }
            },
            move |err| error!("{}", err),
            None,
        )?;

        stream.play()?;

        Ok(Some(OutputStream {
            _stream: stream,
            sender,
        }))
    }
}

impl system::AudioReceiver for AudioReceiver {
    fn queue_sample(&mut self, sample: (i16, i16)) {
        if let Some(stream) = &self.stream {
            stream.sender.send(sample).unwrap();
        }
    }
}
