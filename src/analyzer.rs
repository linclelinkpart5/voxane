use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;

use crate::types::Sample;
use crate::types::Frequency;
use crate::types::SignalStrength;
use crate::spectrum::Spectrum;

pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}

pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

#[derive(Clone)]
pub struct Analyzer {
    buckets: usize,
    window: Vec<Sample>,
    downsample: usize,

    rate: usize,
    lowest: Frequency,
    highest: Frequency,

    fft: Arc<FFT<Sample>>,

    // Defines the target output spectrum.
    spectrum: Spectrum,

    // input: [Vec<rustfft::num_complex::Complex<Sample>>; 2],
    // output: Vec<rustfft::num_complex::Complex<Sample>>,

    // spectra: [Spectrum<Vec<SignalStrength>>; 2],
    // average: Spectrum<Vec<SignalStrength>>,
}

impl Analyzer {
    pub fn new(length: usize, window: Vec<f32>, downsample: usize, rate: usize) -> Self {
        let fft = FFTplanner::new(false).plan_fft(length);
        let buckets = length / 2;

        let downsampled_rate = rate as f32 / downsample as f32;
        let lowest = downsampled_rate / length as f32;
        let highest = downsampled_rate / 2.0;

        let spectrum = Spectrum::new(lowest, highest, length).unwrap();

        Analyzer {
            buckets,
            window,
            downsample,

            rate,
            lowest,
            highest,

            fft,

            spectrum,

            // input: [Vec::with_capacity(length), Vec::with_capacity(length)],
            // output: vec![rustfft::num_complex::Complex::zero(); length],

            // spectra: [
            //     Spectrum::new(vec![0.0; buckets], lowest, highest),
            //     Spectrum::new(vec![0.0; buckets], lowest, highest),
            // ],
            // average: Spectrum::new(vec![0.0; buckets], lowest, highest),
        }
    }

    pub fn len(&self) -> usize {
        self.fft.len()
    }
}
