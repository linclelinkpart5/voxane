use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

use crate::Error;
use crate::sample::Sample;
use crate::types::Frequency;
use crate::types::SignalStrength;
use crate::buckets::Buckets;
use crate::window_kind::WindowKind;

pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}

pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

#[derive(Clone)]
pub struct Analyzer {
    // Reusable FFT algorithm.
    fft: Arc<dyn FFT<Sample>>,

    // FFT window type to use for smoothing.
    window_kind: WindowKind,
}

impl Analyzer {
    pub fn new(len: usize, window_kind: WindowKind) -> Self {
        let fft = FFTplanner::new(false).plan_fft(len);

        Analyzer { fft, window_kind, }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fft.len()
    }

    /// Analyzes a slice of samples, representing a buffer of audio data for one channel.
    /// The sample slice is assumed to be sampled at the same sampling rate as what was used to create this analyzer.
    pub fn calculate_spectrum(&self, samples: &[Sample]) -> Result<Vec<SignalStrength>, Error> {
        // Take enough from the end of the samples to fill the FFT buffer.
        if self.len() != samples.len() { Err(Error::NumSamples(self.len(), samples.len()))? }

        let sample_iter = samples.into_iter().skip(samples.len() - self.len());
        let window_iter = self.window_kind.generate(self.len());

        let mut fft_input_buffer = Vec::with_capacity(self.len());
        let mut fft_output_buffer = vec![Complex::from(0.0); self.len()];

        for (sample_v, window_v) in sample_iter.zip(window_iter) {
            fft_input_buffer.push(Complex::from(sample_v * window_v as f32));
        }

        // The FFT buffer should have the expected number of elements.
        assert_eq!(self.len(), fft_input_buffer.len());

        self.fft.process(fft_input_buffer.as_mut_slice(), fft_output_buffer.as_mut_slice());

        let res =
            fft_output_buffer
                .into_iter()
                // .take(self.len() / 2)
                // .skip(1)
                .map(|o| o.norm_sqr())
                .collect()
        ;

        Ok(res)
    }

    // pub fn bucketize_spectrum(&self, spectrum: &[SignalStrength]) -> Vec<SignalStrength> {
    //     // Using the same unit circle analogy found here: https://dsp.stackexchange.com/q/2970/43899
    //     // The zero index is skipped, since the zero frequency does not apply here.
    //     let valid_fft_indices = 1..=(spectrum.len() / 2);

    //     let mut assignments = vec![(0.0f32, 0); self.buckets.len()];

    //     for i in valid_fft_indices {
    //         let freq_bin = self.fft_bin_size * i as f32;

    //         // Where does this frequency bin fall in the buckets?
    //         if let Some(band_index) = self.buckets.locate(freq_bin) {
    //             if let Some((value, count)) = assignments.get_mut(band_index) {
    //                 *value += spectrum[i];
    //                 *count += 1;
    //             }
    //         }
    //     }

    //     let bucketized =
    //         assignments
    //         .into_iter()
    //         .map(|(value, count)| {
    //             if count > 0 { value / count as f32 }
    //             else { 0.0 }
    //         })
    //         .collect::<Vec<_>>()
    //     ;

    //     assert_eq!(self.buckets.len(), bucketized.len());

    //     bucketized
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    use crate::wave::WaveFunction;
    use crate::wave::WaveGen;

    const NUM_BUCKETS: usize = 16;
    const SAMPLES_PER_PERIOD: usize = 44100;
    const SAMPLE_RATE: Frequency = 44100 as Frequency;
    const FREQUENCY: Frequency = 440.0;

    fn generate_samples(len: usize) -> Vec<Sample> {
        // Generate a wave sample buffer.
        WaveGen::new(WaveFunction::Sine, SAMPLES_PER_PERIOD, FREQUENCY).take(len).collect()
    }

    #[test]
    fn test_calculate_spectrum() {
        const FFT_LEN: usize = 16;

        let analyzer = Analyzer::new(FFT_LEN, WindowKind::Rectangular);

        let samples = generate_samples(FFT_LEN);

        let spectrum: Vec<SignalStrength> = analyzer.calculate_spectrum(&samples).unwrap();

        assert_eq!(FFT_LEN, spectrum.len());

        let expected: Vec<SignalStrength> = vec![
            3.0186355,
            0.31955782,
            0.07949541,
            0.03741721,
            0.023034703,
            0.016638935,
            0.013468596,
            0.011947523,
            0.011491794,
            0.011947523,
            0.013468596,
            0.016638935,
            0.023034703,
            0.03741721,
            0.07949541,
            0.31955782,
        ];

        for (e, ss) in expected.into_iter().zip(&spectrum) {
            assert_approx_eq!(e, ss);
        }

        // let fft_bin_size = SAMPLE_RATE as Frequency / FFT_LEN as f32;

        // for (n, ss) in spectrum.iter().enumerate() {
        //     println!("{}: {} ({} Hz)", n, ss, n as f32 * fft_bin_size);
        // }

        // println!("{:?}", analyzer.buckets());
    }

    // #[test]
    // fn test_bucketize_spectrum() {
    //     const FFT_LEN: usize = 512;

    //     let analyzer = Analyzer::new(FFT_LEN, WindowKind::Rectangular);

    //     let samples = generate_samples(FFT_LEN);

    //     let spectrum: Vec<SignalStrength> = analyzer.calculate_spectrum(&samples).unwrap();
    //     let bucketed_spectrum = analyzer.bucketize_spectrum(&spectrum);

    //     assert_eq!(NUM_BUCKETS, bucketed_spectrum.len());

    //     for (bucket, b_ss) in analyzer.buckets().iter().zip(&bucketed_spectrum) {
    //         println!("[{}, {}): {}", bucket.0, bucket.1, b_ss);
    //     }

    //     // println!("{:?}", analyzer.buckets());
    //     // println!("{:?}", bucketed_spectrum);

    //     // let expected: Vec<SignalStrength> = vec![
    //     //     3.0186355,
    //     //     0.31955782,
    //     //     0.07949541,
    //     //     0.03741721,
    //     //     0.023034703,
    //     //     0.016638935,
    //     //     0.013468596,
    //     //     0.011947523,
    //     //     0.011491794,
    //     //     0.011947523,
    //     //     0.013468596,
    //     //     0.016638935,
    //     //     0.023034703,
    //     //     0.03741721,
    //     //     0.07949541,
    //     //     0.31955782,
    //     // ];

    //     // for (e, ss) in expected.into_iter().zip(&spectrum) {
    //     //     assert_approx_eq!(e, ss);
    //     // }

    //     // let fft_bin_size = SAMPLE_RATE as Frequency / FFT_LEN as f32;

    //     // for (n, ss) in spectrum.iter().enumerate() {
    //     //     println!("{}: {} ({} Hz)", n, ss, n as f32 * fft_bin_size);
    //     // }

    //     // println!("{:?}", analyzer.buckets());
    // }
}
