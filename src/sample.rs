use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::collections::VecDeque;

use crate::types::SignalStrength;

pub type Sample = f32;

pub struct SampleBuffer(Arc<Mutex<VecDeque<Sample>>>);

impl SampleBuffer {
    /// Create a new sample buffer.
    pub fn new(len: usize) -> Self {
        let buffer = VecDeque::from(vec![0.0; len]);
        Self(Arc::new(Mutex::from(buffer)))
    }

    /// Get the length of the buffer.
    /// This should remain constant.
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    /// Push a slice of samples to the buffer.
    pub fn push(&mut self, new: &[Sample]) {
        let mut buf = self.0.lock().unwrap();

        if buf.len() == 0 { return }

        for sample in new.iter() {
            buf.pop_front();
            buf.push_back(*sample);
        }
    }

    // Lock the buffer and return an iterator over its samples.
    pub fn iter<'a>(&'a self) -> SampleIterator<'a> {
        SampleIterator {
            buf: self.0.lock().unwrap(),
            idx: 0,
        }
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

impl<II> From<II> for SampleBuffer
where
    II: IntoIterator<Item = Sample>,
{
    fn from(ii: II) -> Self {
        Self(Arc::new(Mutex::from(ii.into_iter().collect::<VecDeque<_>>())))
    }
}

pub struct SampleIterator<'a> {
    buf: MutexGuard<'a, VecDeque<Sample>>,
    idx: usize,
}

impl Iterator for SampleIterator<'_> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.buf.get(self.idx).copied();
        self.idx += 1;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil as TU;

    #[test]
    fn test_mean_sqr() {
        let inputs_and_expected = vec![
            (SampleBuffer::from(vec![0.0; 16]), 0.0),
            (SampleBuffer::from(vec![1.0; 16]), 1.0),
            (SampleBuffer::from(TU::generate_wave_samples(128, 440.0, 128)), 0.03125002),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.mean_sqr();
            println!("{}, {}", expected, produced);
            assert_approx_eq!(expected, produced);
        }
    }

    #[test]
    fn test_root_mean_sqr() {
        let inputs_and_expected = vec![
            (SampleBuffer::from(vec![0.0; 16]), 0.0),
            (SampleBuffer::from(vec![1.0; 16]), 1.0),
            (SampleBuffer::from(TU::generate_wave_samples(128, 440.0, 128)), 0.17677675),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.root_mean_sqr();
            println!("{}, {}", expected, produced);
            assert_approx_eq!(expected, produced);
        }
    }
}
