use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering as AtomicOrdering;

use cpal::EventLoop;
use cpal::UnknownTypeInputBuffer;
use cpal::StreamData;
use cpal::Sample;

pub struct SoundCard;

impl SoundCard {
    pub fn run() {
        // Setup the default input device and stream with the default input format.
        let device = cpal::default_input_device().expect("failed to get default input device");
        let format = device.default_input_format().expect("failed to get default input format");

        println!("Default input device: {}", device.name());
        println!("Default input format: {:?}", format);

        let event_loop = EventLoop::new();
        let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");
        event_loop.play_stream(stream_id);

        // A flag to indicate that recording is in progress.
        println!("Begin recording...");
        let recording = Arc::new(AtomicBool::new(true));

        // Run the input stream on a separate thread.
        let recording_2 = recording.clone();

        std::thread::spawn(move || {
            event_loop.run(move |_id, data| {
                // If we're done recording, return early.
                if !recording_2.load(AtomicOrdering::Relaxed) {
                    return;
                }
                // Otherwise write to the wav writer.
                match data {
                    StreamData::Input { buffer: UnknownTypeInputBuffer::U16(buffer) } => {
                        for sample in buffer.iter() {
                            let sample = Sample::to_f32(sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::I16(buffer) } => {
                        for &sample in buffer.iter() {
                            let sample = Sample::to_f32(&sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                        for &sample in buffer.iter() {
                            let sample = Sample::to_f32(&sample);
                        }
                    },
                    _ => (),
                }
            });
        });
    }
}
