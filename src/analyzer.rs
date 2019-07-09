use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

use crate::Error;
use crate::sample::Sample;
use crate::types::Frequency;
use crate::types::SignalStrength;
use crate::buckets::Buckets;
use crate::window::Window;

pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}

pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

#[derive(Clone)]
pub struct Analyzer {
    // Reusable FFT algorithm.
    fft: Arc<dyn FFT<Sample>>,

    // FFT frequency resolution, i.e. how far apart consecutive FFT bins are from each other.
    fft_bin_size: Frequency,

    // FFT window type to use for smoothing.
    window: Window,

    // Defines the target output frequency buckets.
    buckets: Buckets,

    // Skip this many samples between processing each sample.
    // downsample_skip: usize,

    // input: [Vec<rustfft::num_complex::Complex<Sample>>; 2],
    // output: Vec<rustfft::num_complex::Complex<Sample>>,

    // spectra: [Spectrum<Vec<SignalStrength>>; 2],
    // average: Spectrum<Vec<SignalStrength>>,
}

impl Analyzer {
    pub fn new(
        fft_buffer_len: usize,
        bucket_len: usize,
        window: Window,
        lower_cutoff: Frequency,
        upper_cutoff: Frequency,
        sampling_rate: Frequency,
    ) -> Result<Self, Error>
    {
        if !(sampling_rate > 0.0) { Err(Error::InvalidSamplingRate)? }

        // Force upper cutoff frequency to be no higher than half of the sampling rate.
        let upper_cutoff = upper_cutoff.min(sampling_rate / 2.0);

        let buckets = Buckets::new(lower_cutoff, upper_cutoff, bucket_len)?;

        let fft = FFTplanner::new(false).plan_fft(fft_buffer_len);
        let fft_bin_size = sampling_rate / fft_buffer_len as f32;

        Ok(Analyzer {
            fft,
            fft_bin_size,
            window,
            buckets,
        })
    }

    #[inline]
    pub fn fft_buffer_len(&self) -> usize {
        self.fft.len()
    }

    #[inline]
    pub fn buckets_len(&self) -> usize {
        self.buckets.len()
    }

    #[inline]
    pub fn buckets(&self) -> &[(Frequency, Frequency)] {
        self.buckets.bands()
    }

    #[inline]
    pub fn lower_cutoff(&self) -> Option<Frequency> {
        self.buckets.lower_cutoff()
    }

    #[inline]
    pub fn upper_cutoff(&self) -> Option<Frequency> {
        self.buckets.upper_cutoff()
    }

    /// Analyzes a slice of samples, representing a buffer of audio data for one channel.
    /// The sample slice is assumed to be sampled at the same sampling rate as what was used to create this analyzer.
    pub fn calculate_spectrum(&self, samples: &[Sample]) -> Result<Vec<SignalStrength>, Error> {
        // Take enough from the end of the samples to fill the FFT buffer.
        if !(samples.len() >= self.fft_buffer_len()) { Err(Error::NotEnoughSamples)? }

        let sample_iter = samples.into_iter().skip(samples.len() - self.fft_buffer_len());
        let window_iter = self.window.generate(self.fft_buffer_len());

        let mut fft_input_buffer = Vec::with_capacity(self.fft_buffer_len());
        let mut fft_output_buffer = vec![Complex::from(0.0); self.fft_buffer_len()];

        for (sample_v, window_v) in sample_iter.zip(window_iter) {
            fft_input_buffer.push(Complex::from(sample_v * window_v as f32));
        }

        // The FFT buffer should have the expected number of elements.
        assert_eq!(self.fft_buffer_len(), fft_input_buffer.len());

        self.fft.process(fft_input_buffer.as_mut_slice(), fft_output_buffer.as_mut_slice());

        let res =
            fft_output_buffer
                .into_iter()
                // .take(self.fft_buffer_len() / 2)
                // .skip(1)
                .map(|o| o.norm_sqr())
                .collect()
        ;

        Ok(res)
    }

    pub fn calculate_bucketed_spectrum(&self, samples: &[Sample]) -> Result<Vec<SignalStrength>, Error> {
        let spectrum = self.calculate_spectrum(samples)?;

        // Using the same unit circle analogy found here: https://dsp.stackexchange.com/q/2970/43899
        // The zero index is skipped, since the zero frequency does not apply here.
        let valid_fft_indices = 1..=(spectrum.len() / 2);

        let mut assignments = vec![(0.0f32, 0); self.buckets.len()];

        for i in valid_fft_indices {
            let freq_bin = self.fft_bin_size * i as f32;

            // Where does this frequency bin fall in the buckets?
            if let Some(band_index) = self.buckets.locate(freq_bin) {
                if let Some((value, count)) = assignments.get_mut(band_index) {
                    *value += spectrum[i];
                    *count += 1;
                }
            }
        }

        let band_values =
            assignments
            .into_iter()
            .map(|(value, count)| {
                if count > 0 { value / count as f32 }
                else { 0.0 }
            })
            .collect::<Vec<_>>()
        ;

        assert_eq!(self.buckets.len(), band_values.len());

        Ok(band_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::wave::WaveFunction;
    use crate::wave::WaveGen;

    #[test]
    fn test_calculate_spectrum() {
        const FFT_LEN: usize = 512;
        const NUM_BUCKETS: usize = 16;
        const SAMPLE_RATE: usize = 44100;

        let analyzer = Analyzer::new(FFT_LEN, NUM_BUCKETS, Window::Rectangle, 20.0, 10000.0, SAMPLE_RATE as Frequency).unwrap();

        // let samples: Vec<Sample> = vec![1.0; 256];

        // Generate a wave sample buffer.
        let samples: Vec<Sample> = WaveGen::new(WaveFunction::Sine, SAMPLE_RATE).take(FFT_LEN).collect();

        let spectrum: Vec<SignalStrength> = analyzer.calculate_spectrum(&samples).unwrap();

        let fft_bin_size = SAMPLE_RATE as Frequency / FFT_LEN as f32;

        for (n, ss) in spectrum.iter().enumerate() {
            println!("{}: {} ({} Hz)", n, ss, n as f32 * fft_bin_size);
        }

        // println!("{:?}", signal_strength);
        println!("{:?}", analyzer.buckets());
    }
}
