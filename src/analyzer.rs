use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

use crate::Error;
use crate::types::Sample;
use crate::types::Frequency;
use crate::types::SignalStrength;
use crate::spectrum::Spectrum;
use crate::window::Window;

pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}

pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

#[derive(Clone)]
pub struct Analyzer {
    // Reusable FFT algorithm.
    fft: Arc<FFT<Sample>>,

    // FFT window type to use for smoothing.
    window: Window,

    // Defines the target output spectrum.
    spectrum: Spectrum,

    // input: [Vec<rustfft::num_complex::Complex<Sample>>; 2],
    // output: Vec<rustfft::num_complex::Complex<Sample>>,

    // spectra: [Spectrum<Vec<SignalStrength>>; 2],
    // average: Spectrum<Vec<SignalStrength>>,
}

impl Analyzer {
    pub fn new(
        fft_buffer_size: usize,
        spectrum_size: usize,
        window: Window,
        lower_cutoff: Frequency,
        upper_cutoff: Frequency,
        sampling_rate: Frequency,
    ) -> Result<Self, Error>
    {
        if !(sampling_rate > 0.0) { Err(Error::InvalidSamplingRate)? }

        // Force upper cutoff frequency to be no higher than half of the sampling rate.
        let upper_cutoff = upper_cutoff.min(sampling_rate / 2.0);

        let spectrum = Spectrum::new(lower_cutoff, upper_cutoff, spectrum_size)?;

        let fft = FFTplanner::new(false).plan_fft(fft_buffer_size);

        Ok(Analyzer {
            fft,
            window,
            spectrum,
        })
    }

    #[inline]
    pub fn fft_buffer_size(&self) -> usize {
        self.fft.len()
    }

    #[inline]
    pub fn spectrum_size(&self) -> usize {
        self.spectrum.num_bands()
    }

    #[inline]
    pub fn lower_cutoff(&self) -> Option<Frequency> {
        self.spectrum.lower_cutoff()
    }

    #[inline]
    pub fn upper_cutoff(&self) -> Option<Frequency> {
        self.spectrum.upper_cutoff()
    }

    /// Analyzes a slice of samples, representing a buffer of audio data for one channel.
    /// The sample slice is assumed to be sampled at the same frequency as what was used to create this analyzer.
    pub fn analyze(&self, samples: &[Sample]) -> Vec<SignalStrength> {
        // Take enough from the end of the samples to fill the FFT buffer.
        assert!(samples.len() >= self.fft_buffer_size());

        let sample_iter = samples.into_iter().skip(samples.len() - self.fft_buffer_size());
        let window_iter = self.window.generate(self.fft_buffer_size());

        let mut fft_input_buffer = Vec::with_capacity(self.fft_buffer_size());
        let mut fft_output_buffer = vec![Complex::from(0.0); self.fft_buffer_size()];

        for (sample_v, window_v) in sample_iter.zip(window_iter) {
            fft_input_buffer.push(Complex::from(sample_v * window_v as f32));
        }

        // The FFT buffer should have the expected number of elements.
        assert_eq!(self.fft_buffer_size(), fft_input_buffer.len());

        self.fft.process(fft_input_buffer.as_mut_slice(), fft_output_buffer.as_mut_slice());

        fft_output_buffer.into_iter().map(|o| o.norm_sqr()).collect()
    }
}
