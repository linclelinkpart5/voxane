//! Reads audio data in-transit and pushes samples into a buffer.

use std::sync::Arc;
use std::sync::Mutex;
use std::thread::Builder as ThreadBuilder;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;

use cpal::UnknownTypeInputBuffer;
use cpal::StreamData;
use cpal::SampleRate;
use cpal::SampleFormat;
use cpal::Format;
use cpal::traits::HostTrait;
// use cpal::traits::DeviceTrait;
use cpal::traits::EventLoopTrait;

use crate::sample::SampleSink;
use crate::sample::SampleBuffer;

const NUM_CHANNELS: u16 = 2;

pub struct Listener {
    sample_sink: SampleSink,
    sample_rate: usize,
    read_size: usize,
    is_running: Arc<AtomicBool>,
}

impl Listener {
    pub fn start(sample_rate: usize, buffer_len: usize, read_size: usize) -> Self {
        let sample_sink = Arc::new(Mutex::from(SampleBuffer::new(buffer_len)));
        let is_running = Arc::new(AtomicBool::from(true));

        // Scope for thread spawning.
        {
            let sample_sink = sample_sink.clone();
            let is_running = is_running.clone();

            // This is a smaller buffer for shuttling data,
            // in order to keep the sample sink from being locked for too long.
            let transport_buffer = vec![0.0; read_size * NUM_CHANNELS as usize];

            ThreadBuilder::new()
                .spawn(move || {
                    // Set up host, device, format, and stream.
                    let host = cpal::default_host();

                    let format = Format {
                        channels: NUM_CHANNELS,
                        sample_rate: SampleRate(sample_rate as _),
                        data_type: SampleFormat::F32,
                    };

                    let event_loop = host.event_loop();

                    let device = host.default_input_device().expect("failed to get default input device");

                    let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");

                    event_loop.play_stream(stream_id).unwrap();

                    // Run the event loop and fill sample sink.
                    event_loop.run(|_stream_id, stream_result| {
                        // Check to see if a stop is requested.
                        if !is_running.load(AtomicOrdering::Relaxed) { return }

                        let stream_data = stream_result.unwrap();

                        match stream_data {
                            StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                                // println!("CPAL buffer size: {}", buffer.len());
                                for chunk in buffer.chunks(transport_buffer.len()) {
                                    let mut sample_buffer = sample_sink.lock().unwrap();
                                    sample_buffer.push_interleaved(&chunk);
                                }
                            },
                            StreamData::Input { .. } => panic!(),
                            StreamData::Output { .. } => {},
                        };
                    });
                })
                .unwrap()
            ;
        }

        Self {
            sample_sink,
            sample_rate,
            read_size,
            is_running,
        }
    }

    pub fn stop(&self) {
        self.is_running.store(false, AtomicOrdering::Relaxed);
    }

    pub fn sample_sink<'a>(&'a self) -> &'a SampleSink {
        &self.sample_sink
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    #[test]
    fn test_listen() {
        let listener = Listener::start(44100, 8192, 256);

        // listener.start();

        std::thread::sleep(Duration::from_secs(5));

        listener.stop();

        let sample_sink = listener.sample_sink();

        let sample_buffer = sample_sink.lock().unwrap();

        for (l_sample, r_sample) in sample_buffer.iter().take(16) {
            println!("({}, {})", l_sample, r_sample);
        }
    }
}
