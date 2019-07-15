use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use crate::Error;
use crate::sample::Sample;
use crate::types::SignalStrength;
use crate::window_kind::WindowKind;

#[derive(Clone)]
pub struct Analyzer {
    // Reusable FFT algorithm.
    fft: Arc<dyn FFT<Sample>>,

    // FFT window to use for smoothing.
    window: Vec<f32>,

    // Intermediate FFT buffers.
    input: Vec<Complex<Sample>>,
    output: Vec<Complex<Sample>>,

    spectrum: Vec<SignalStrength>,
}

impl Analyzer {
    pub fn new(len: usize, window_kind: WindowKind) -> Self {
        let fft = FFTplanner::new(false).plan_fft(len);

        let window = window_kind.generate(len).into_iter().map(|w| w as f32).collect();

        let input = vec![Complex::zero(); len];
        let output = vec![Complex::zero(); len];

        let spectrum = vec![0.0; len];

        Analyzer {
            fft,
            window,
            input,
            output,
            spectrum,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fft.len()
    }

    /// Analyzes a sample buffer, representing a buffer of audio data for one channel.
    pub fn analyze(&mut self, samples: &[Sample]) -> Result<Vec<SignalStrength>, Error> {
        // Check to see if the number of samples is correct.
        if self.len() != samples.len() { Err(Error::NumSamples(self.len(), samples.len()))? }

        for (i, (x, w)) in self.input.iter_mut().zip(samples.iter().zip(&self.window)) {
            *i = Complex::new(x * w, 0.0);
        }

        self.fft.process(&mut self.input, &mut self.output);

        for (s, o) in self.spectrum.iter_mut().zip(&self.output) {
            *s = o.norm_sqr();
        }

        Ok(self.spectrum.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil;
    use crate::types::Frequency;

    const SAMPLES_PER_PERIOD: usize = 44100;
    const FREQUENCY: Frequency = 440.0;

    #[test]
    fn test_analyze() {
        const FFT_LEN: usize = 16;

        let mut analyzer = Analyzer::new(FFT_LEN, WindowKind::Rectangular);

        let samples = TestUtil::generate_wave_samples(SAMPLES_PER_PERIOD, FREQUENCY, FFT_LEN);

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

    #[test]
    fn test_analyze_large_window() {
        const FFT_LEN: usize = 2048;

        let mut analyzer = Analyzer::new(FFT_LEN, WindowKind::Rectangular);

        let samples = TestUtil::generate_wave_samples(SAMPLES_PER_PERIOD, FREQUENCY, FFT_LEN);

        let spectrum: Vec<SignalStrength> = analyzer.analyze(&samples).unwrap();

        assert_eq!(FFT_LEN, spectrum.len());

        let fft_bin_size = SAMPLES_PER_PERIOD as Frequency / FFT_LEN as f32;

        let expected_max_spectrum_index = (FREQUENCY / fft_bin_size) as usize;

        let produced_max_spectrum_index =
            spectrum
            .iter()
            .take(FFT_LEN / 2)
            .enumerate()
            .max_by(|(_, sa), (_, sb)| sa.partial_cmp(&sb).unwrap())
            .map(|(i, _)| i)
            .unwrap()
        ;

        println!("{}, {}", expected_max_spectrum_index, produced_max_spectrum_index);
        assert_eq!(expected_max_spectrum_index, produced_max_spectrum_index);

        // for (n, ss) in spectrum.iter().enumerate() {
        //     println!("{}: {} ({} Hz)", n, ss, n as f32 * fft_bin_size);
        // }
    }
}
