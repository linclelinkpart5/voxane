//! Reads audio data in-transit and pushes samples into a buffer.

use std::sync::Arc;
use std::thread::Builder as ThreadBuilder;
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;

use cpal::UnknownTypeInputBuffer;
use cpal::EventLoop;
use cpal::StreamId;
use cpal::StreamData;
use cpal::SampleRate;
use cpal::SampleFormat;
use cpal::Format;
use cpal::traits::HostTrait;
use cpal::traits::DeviceTrait;
use cpal::traits::EventLoopTrait;

use crate::sample::SampleBuffer;

const NUM_CHANNELS: u16 = 2;

pub struct Listener {
    sample_buffer: SampleBuffer,
    stream_id: StreamId,
    event_loop: Arc<EventLoop>,
}

impl Listener {
    pub fn new(sample_rate: usize, buffer_len: usize, read_size: usize) -> Self {
        let sample_buffer = SampleBuffer::new(buffer_len);

        // Set up host, device, format, and stream.
        let host = cpal::default_host();

        for d in host.output_devices().unwrap() {
            println!("{:?}", d.name());
        }

        let format = Format {
            channels: NUM_CHANNELS,
            sample_rate: SampleRate(sample_rate as _),
            data_type: SampleFormat::F32,
        };

        let event_loop = Arc::new(host.event_loop());

        let device = host.default_output_device().expect("failed to get default output device");

        let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");

        // Scope for thread spawning.
        let _handle = {
            // Since these are both using `Arc` to some degree, the calls to clone are cheap.
            let mut sample_buffer = sample_buffer.clone();
            let event_loop = event_loop.clone();

            // This is a smaller buffer for shuttling data,
            // in order to keep the sample sink from being locked for too long.
            // let transport_buffer = vec![0.0; read_size * NUM_CHANNELS as usize];
            let transport_size = read_size * NUM_CHANNELS as usize;

            ThreadBuilder::new()
                .spawn(move || {
                    // Run the event loop and fill sample sink.
                    event_loop.run(|_stream_id, stream_result| {
                        // println!("{:?}", _stream_id);
                        let stream_data = stream_result.unwrap();

                        match stream_data {
                            StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                                // println!("CPAL buffer size: {}", buffer.len());
                                for chunk in buffer.chunks(transport_size) {
                                    sample_buffer.push_interleaved(&chunk);
                                }
                            },
                            StreamData::Input { .. } => panic!("unrequested format"),
                            StreamData::Output { .. } => {},
                        };
                    });
                })
                .unwrap()
        };

        Self {
            sample_buffer,
            stream_id,
            event_loop,
        }
    }

    pub fn play(&self) {
        // print!("PLAYING... ");
        self.event_loop.play_stream(self.stream_id.clone()).unwrap();
        // println!("PLAYED");
    }

    pub fn pause(&self) {
        // print!("PAUSING... ");
        self.event_loop.pause_stream(self.stream_id.clone()).unwrap();
        // println!("PAUSED");
    }

    pub fn sample_buffer<'a>(&'a self) -> &'a SampleBuffer {
        &self.sample_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;

    #[test]
    fn test_listen() {
        let mut output_file = File::create("wav_data.csv").unwrap();

        let listener = Listener::new(44100, 44100*3, 256);

        let sample_buffer = listener.sample_buffer().clone();

        listener.play();

        std::thread::sleep(Duration::from_secs(5));

        for (l_sample, r_sample) in sample_buffer.iter() {
            writeln!(output_file, "{},{}", l_sample, r_sample).unwrap();
        }

        std::thread::sleep(Duration::from_secs(5));

        for (l_sample, r_sample) in sample_buffer.iter() {
            writeln!(output_file, "{},{}", l_sample, r_sample).unwrap();
        }

        listener.pause();
    }
}
