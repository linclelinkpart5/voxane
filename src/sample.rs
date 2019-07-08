use std::collections::VecDeque;

use crate::types::SignalStrength;

pub type Sample = f32;

pub struct SampleBuffer(VecDeque<Sample>);

impl SampleBuffer {
    /// Create a new empty sample buffer given a size.
    pub fn new(size: usize) -> Self {
        let buffer = VecDeque::from(vec![0.0; size]);
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

    pub fn iter_tail(&self) -> impl Iterator<Item = &Sample> {
        self.0.iter().rev()
    }

    // TODO: Create `.volume()` method on future enum that abstracts number of channels.
}
