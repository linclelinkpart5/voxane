use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::collections::VecDeque;

use crate::types::SignalStrength;

pub type Sample = f32;

pub struct SampleBuffer(VecDeque<(Sample, Sample)>);

impl SampleBuffer {
    /// Create a new sample buffer.
    pub fn new(len: usize) -> Self {
        let buffer = VecDeque::from(vec![(0.0, 0.0); len]);
        Self(buffer)
    }

    /// Get the length of the buffer.
    /// This should remain constant.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Push a slice of stereo samples to the buffer.
    pub fn push(&mut self, sample_pairs: &[(Sample, Sample)]) {
        if self.0.len() == 0 { return }

        for sample_pair in sample_pairs.iter() {
            self.0.pop_front();
            self.0.push_back(*sample_pair);
        }
    }

    /// Push a slice of interleaved samples to the buffer.
    pub fn push_interleaved(&mut self, samples: &[Sample]) {
        if self.0.len() == 0 { return }

        for sample_chunk in samples.chunks_exact(2) {
            self.0.pop_front();
            self.0.push_back((sample_chunk[0], sample_chunk[1]));
        }
    }

    /// Return an iterator over the stereo samples in this buffer.
    pub fn iter(&self) -> impl Iterator<Item = &(Sample, Sample)> {
        self.0.iter()
    }

    ///
    pub fn rms(&self) -> SignalStrength {
        // Taken from http://replaygain.hydrogenaud.io/proposal/rms_energy.html
        if self.len() == 0 { return 0.0 }

        let s_pair =
            self
            .iter()
            .map(|(l, r)| (l.powi(2), r.powi(2)))
            .fold((0.0, 0.0), |(al, ar), (l, r)| (al + l, ar + r))
        ;

        let n = self.len() as SignalStrength;
        let ms_pair = (s_pair.0 / n, s_pair.1 / n);

        ((ms_pair.0 + ms_pair.1) / 2.0).sqrt()
    }
}

impl From<Vec<(Sample, Sample)>> for SampleBuffer {
    fn from(v: Vec<(Sample, Sample)>) -> Self {
        Self(v.into())
    }
}

impl From<Vec<Sample>> for SampleBuffer {
    fn from(v: Vec<Sample>) -> Self {
        Self(v.into_iter().map(|s| (s, s)).collect())
    }
}

pub type SampleSink = Arc<Mutex<SampleBuffer>>;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil as TU;

    #[test]
    fn test_root_mean_sqr() {
        let inputs_and_expected = vec![
            (SampleBuffer::from(vec![0.0; 16]), 0.0),
            (SampleBuffer::from(vec![1.0; 16]), 1.0),
            (SampleBuffer::from(TU::generate_wave_samples(128, 440.0, 128)), 0.17677675),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.rms();
            println!("{}, {}", expected, produced);
            assert_approx_eq!(expected, produced);
        }
    }
}
