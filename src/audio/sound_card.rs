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

pub struct SoundCard {
    event_loop: EventLoop,
}

impl SoundCard {
    pub fn new() -> Self {
        // Setup the default input device and stream with the default input format.
        let host = cpal::default_host();

        let event_loop = host.event_loop();

        let device = host.default_input_device().expect("failed to get default input device");
        let format = device.default_input_format().expect("failed to get default input format");
        let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");
        event_loop.play_stream(stream_id).expect("unable to play input stream");

        Self {
            event_loop,
        }
    }

    pub fn run() {
        // Setup the default input device and stream with the default input format.
        let host = cpal::default_host();

        let event_loop = host.event_loop();

        let device = host.default_input_device().expect("failed to get default input device");
        let format = device.default_input_format().expect("failed to get default input format");
        let stream_id = event_loop.build_input_stream(&device, &format).expect("failed to build input stream");
        event_loop.play_stream(stream_id).expect("unable to play input stream");

        // A flag to indicate that recording is in progress.
        println!("Begin recording...");
        let recording = Arc::new(AtomicBool::new(true));

        // Run the input stream on a separate thread.
        let recording_2 = recording.clone();

        std::thread::spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                // If we're done recording, return early.
                if !recording_2.load(AtomicOrdering::Relaxed) {
                    return;
                }

                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                    _ => return,
                };

                // Otherwise write to the wav writer.
                match stream_data {
                    StreamData::Input { buffer: UnknownTypeInputBuffer::U16(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for sample in buffer.iter() {
                            let sample = Sample::to_f32(sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::I16(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for &sample in buffer.iter() {
                            let sample = Sample::to_f32(&sample);
                        }
                    },
                    StreamData::Input { buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                        println!("Buffer size: {}", buffer.len());
                        for &sample in buffer.iter() {
                            let sample = Sample::to_f32(&sample);
                        }
                    },
                    _ => (),
                }
            });
        });

        loop {}
    }
}

#[cfg(test)]
mod tests {
    use super::SoundCard;

    #[test]
    fn test_run() {
        SoundCard::run();
    }
}
