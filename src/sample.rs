use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::collections::VecDeque;

use crate::Error;
use crate::types::SignalStrength;

pub type Sample = f32;

#[derive(Clone)]
pub struct SampleBuffer(Arc<Mutex<VecDeque<(Sample, Sample)>>>);

impl SampleBuffer {
    /// Create a new sample buffer.
    pub fn new(len: usize) -> Self {
        let buffer = VecDeque::from(vec![(0.0, 0.0); len]);
        Self(Arc::new(Mutex::from(buffer)))
    }

    /// Get the length of the buffer.
    /// This should remain constant.
    pub fn len(&self) -> usize {
        let buffer = self.0.lock().unwrap();
        buffer.len()
    }

    /// Push a slice of stereo samples to the buffer.
    pub fn push(&mut self, sample_pairs: &[(Sample, Sample)]) {
        let mut buffer = self.0.lock().unwrap();

        if buffer.len() == 0 { return }

        for sample_pair in sample_pairs.iter() {
            buffer.pop_front();
            buffer.push_back(*sample_pair);
        }
    }

    /// Push a slice of interleaved samples to the buffer.
    pub fn push_interleaved(&mut self, samples: &[Sample]) {
        let mut buffer = self.0.lock().unwrap();

        if buffer.len() == 0 { return }

        for sample_chunk in samples.chunks_exact(2) {
            buffer.pop_front();
            buffer.push_back((sample_chunk[0], sample_chunk[1]));
        }
    }

    fn iter_starting_at<'a>(&'a self, n: usize) -> SampleBufferIter<'a> {
        let buffer = self.0.lock().unwrap();

        SampleBufferIter {
            buffer,
            index: n,
        }
    }

    /// Return an iterator over the samples in this buffer.
    pub fn iter<'a>(&'a self) -> SampleBufferIter<'a> {
        self.iter_starting_at(0)
    }

    /// Return an iterator over the last N samples in this buffer.
    pub fn iter_tail<'a>(&'a self, n: usize) -> Result<SampleBufferIter<'a>, Error> {
        let buffer = self.0.lock().unwrap();

        if buffer.len() < n { Err(Error::TooFewSamples(n, buffer.len()))? }

        let start_index = buffer.len() - n;

        Ok(SampleBufferIter {
            buffer,
            index: start_index,
        })
    }
}

impl From<Vec<(Sample, Sample)>> for SampleBuffer {
    fn from(v: Vec<(Sample, Sample)>) -> Self {
        let buffer: VecDeque<_> = v.into();
        Self(Arc::new(Mutex::from(buffer)))
    }
}

impl From<Vec<Sample>> for SampleBuffer {
    fn from(v: Vec<Sample>) -> Self {
        Self(Arc::new(Mutex::from(v.into_iter().map(|s| (s, s)).collect::<VecDeque<_>>())))
    }
}

pub struct SampleBufferIter<'a> {
    buffer: MutexGuard<'a, VecDeque<(Sample, Sample)>>,
    index: usize,
}

impl<'a> SampleBufferIter<'a> {
    /// Calculates the RMS of the samples in this iterator.
    pub fn rms(mut self) -> SignalStrength {
        match self.next() {
            None => 0.0,
            Some((il, ir)) => {
                let initial_state = (il.powi(2), ir.powi(2), 1);
                let (sl, sr, n) =
                    self
                    .map(|(l, r)| (l.powi(2), r.powi(2)))
                    .fold(initial_state, |(al, ar, c), (l, r)| (al + l, ar + r, c + 1))
                ;

                let (msl, msr) = (sl / n as SignalStrength, sr / n as SignalStrength);

                ((msl + msr) / 2.0).sqrt()
            }
        }
    }
}

impl Iterator for SampleBufferIter<'_> {
    type Item = (Sample, Sample);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.buffer.get(self.index).copied();
        self.index += 1;
        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }
}

impl ExactSizeIterator for SampleBufferIter<'_> {
    fn len(&self) -> usize {
        self.buffer.len().checked_sub(self.index).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil as TU;

    #[test]
    fn test_rms() {
        let inputs_and_expected = vec![
            (SampleBuffer::from(vec![0.0; 16]), 0.0),
            (SampleBuffer::from(vec![1.0; 16]), 1.0),
            (SampleBuffer::from(TU::generate_wave_samples(128, 440.0, 128)), 0.17677675),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.iter().rms();
            println!("{}, {}", expected, produced);
            assert_approx_eq!(expected, produced);
        }
    }
}
