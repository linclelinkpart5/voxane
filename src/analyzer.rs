use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

use crate::Error;
use crate::sample::Sample;
use crate::types::Frequency;
use crate::types::SignalStrength;
use crate::window_kind::WindowKind;

#[derive(Clone)]
pub struct Analyzer {
    // Reusable FFT algorithm.
    fft: Arc<dyn FFT<Sample>>,

    // FFT window to use for smoothing.
    window: Vec<f32>,
}

impl Analyzer {
    pub fn new(len: usize, window_kind: WindowKind) -> Self {
        let fft = FFTplanner::new(false).plan_fft(len);

        let window = window_kind.generate(len).into_iter().map(|w| w as f32).collect();

        Analyzer { fft, window, }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fft.len()
    }

    /// Analyzes a sample buffer, representing a buffer of audio data for one channel.
    pub fn analyze(&self, samples: &[Sample]) -> Result<Vec<SignalStrength>, Error> {
        // Check to see if the number of samples is correct.
        if self.len() != samples.len() { Err(Error::NumSamples(self.len(), samples.len()))? }

        let mut fft_input_buffer =
            samples
            .iter()
            .zip(&self.window)
            .map(|(s, w)| Complex::new(s * w, 0.0))
            .collect::<Vec<_>>()
        ;
        let mut fft_output_buffer = vec![Complex::from(0.0); self.len()];

        self.fft.process(fft_input_buffer.as_mut_slice(), fft_output_buffer.as_mut_slice());

        let res =
            fft_output_buffer
                .into_iter()
                .map(|o| o.norm_sqr())
                .collect()
        ;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    use crate::wave::WaveFunction;
    use crate::wave::WaveGen;

    const SAMPLES_PER_PERIOD: usize = 44100;
    const FREQUENCY: Frequency = 440.0;

    fn generate_samples(len: usize) -> Vec<Sample> {
        // Generate a wave sample buffer.
        WaveGen::new(WaveFunction::Sine, SAMPLES_PER_PERIOD, FREQUENCY).take(len).collect()
    }

    #[test]
    fn test_analyze() {
        const FFT_LEN: usize = 16;

        let analyzer = Analyzer::new(FFT_LEN, WindowKind::Rectangular);

        let samples = generate_samples(FFT_LEN);

        let spectrum: Vec<SignalStrength> = analyzer.analyze(&samples).unwrap();

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

    //     let spectrum: Vec<SignalStrength> = analyzer.analyze(&samples).unwrap();
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
