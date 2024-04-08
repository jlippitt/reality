use rodio::buffer::SamplesBuffer;
use rodio::queue::{self, SourcesQueueInput};
use rodio::{OutputStream, OutputStreamHandle, Source};
use std::error::Error;
use std::sync::Arc;

const BUFFER_SIZE: usize = 2048;

pub struct AudioReceiver {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    queue_input: Arc<SourcesQueueInput<u16>>,
    samples: Vec<u16>,
    sample_rate: u32,
}

impl AudioReceiver {
    pub fn new(sample_rate: u32) -> Result<AudioReceiver, Box<dyn Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        println!("Here");

        let (queue_input, queue_output) = queue::queue::<u16>(true);
        println!("Still here");

        stream_handle.play_raw(queue_output.convert_samples())?;
        println!("Still still here");

        Ok(AudioReceiver {
            _stream,
            _stream_handle: stream_handle,
            queue_input,
            samples: Vec::with_capacity(BUFFER_SIZE),
            sample_rate,
        })
    }
}

impl system::AudioReceiver for AudioReceiver {
    fn queue_samples(&mut self, sample_rate: u32, samples: &[u16]) {
        if self.samples.len() >= BUFFER_SIZE
            || (sample_rate != self.sample_rate && !self.samples.is_empty())
        {
            let buffer = SamplesBuffer::new(2, sample_rate, self.samples.split_off(0));
            self.queue_input.append(buffer);
        }

        self.samples.extend_from_slice(samples);
        self.sample_rate = sample_rate;
    }
}
