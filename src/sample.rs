use std::collections::VecDeque;

use crate::types::SignalStrength;

pub type Sample = f32;

pub struct SampleBuffer(VecDeque<Sample>);

impl SampleBuffer {
    /// Create a new sample buffer.
    pub fn new(len: usize) -> Self {
        let buffer = VecDeque::from(vec![0.0; len]);
        Self(buffer)
    }

    /// Get the length of the buffer.
    /// This should remain constant.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Push a slice of samples to the buffer.
    pub fn push(&mut self, new: &[Sample]) {
        if self.0.len() == 0 { return }

        for sample in new.iter() {
            self.0.pop_front();
            self.0.push_back(*sample);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sample> {
        self.0.iter()
    }

    pub fn mean_sqr(&self) -> SignalStrength {
        if self.len() == 0 { return 0.0 }

        // Taken from http://replaygain.hydrogenaud.io/proposal/rms_energy.html
        self.iter().map(|v| v.powi(2)).sum::<SignalStrength>() / self.len() as SignalStrength
    }

    pub fn root_mean_sqr(&self) -> SignalStrength {
        self.mean_sqr().sqrt()
    }
}
