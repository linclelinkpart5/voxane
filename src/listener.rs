//! Reads audio data in-transit and pushes samples into a buffer.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;

use cpal::EventLoop;
use cpal::UnknownTypeInputBuffer;
use cpal::StreamData;
use cpal::Sample;
use cpal::traits::HostTrait;
use cpal::traits::DeviceTrait;
use cpal::traits::EventLoopTrait;

use crate::sample::SampleSink;

pub struct Listener {
    event_loop: EventLoop,
    is_running: Arc<AtomicBool>,
    sample_sink: SampleSink,
}

impl Listener {
    pub fn new(sample_sink: SampleSink) -> Self {
        // Setup the default input device and stream with the default input format.
        let host = cpal::default_host();

        let event_loop = host.event_loop();

        let device = host.default_input_device().expect("failed to get default input device");
        let format = device.default_input_format().expect("failed to get default input format");
        let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");
        event_loop.play_stream(stream_id).expect("unable to play input stream");

        Self {
            event_loop,
            is_running: Arc::new(AtomicBool::from(false)),
            sample_sink,
        }
    }

    pub fn stop(&'static mut self) {
        self.is_running.store(false, AtomicOrdering::Relaxed);
    }

    pub fn start(&'static mut self) {
        self.is_running.store(true, AtomicOrdering::Relaxed);

        let is_running_inner = self.is_running.clone();
        let sample_sink_inner = self.sample_sink.clone();

        std::thread::spawn(move || {
            self.event_loop.run(move |stream_id, stream_result| {
                // If we're done running, return early.
                if !is_running_inner.load(AtomicOrdering::Relaxed) {
                    return;
                }

                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };

                // Otherwise write to the sample sink.
                match stream_data {
                    StreamData::Input { buffer: UnknownTypeInputBuffer::U16(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for sample in buffer.iter() {
                            let _sample = Sample::to_f32(sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::I16(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for &sample in buffer.iter() {
                            let _sample = Sample::to_f32(&sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for &sample in buffer.iter() {
                            let _sample = Sample::to_f32(&sample);
                        }
                    },
                    _ => {
                        eprintln!("unexpected sample format");
                        return;
                    },
                }
            });
        });
    }
}
