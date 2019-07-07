use rustfft::num_complex::Complex;

use crate::Error;
use crate::spectrum::Spectrum;

pub struct Assigner {
    spectrum: Spectrum,
    sampling_freq: f32,
}

impl Assigner {
    pub fn new(lower_cutoff_freq: f32, upper_cutoff_freq: f32, num_bands: usize, sampling_freq: f32) -> Result<Self, Error> {
        if !(sampling_freq > 0.0) { Err(Error::InvalidSamplingRate)? }

        let upper_cutoff_freq = upper_cutoff_freq.min(sampling_freq / 2.0);

        let spectrum = Spectrum::new(lower_cutoff_freq, upper_cutoff_freq, num_bands)?;

        Ok(Self{ spectrum, sampling_freq, })
    }

    pub fn assign_fft(&self, fft_output: &[Complex<f32>]) -> Vec<f32> {
        assert!(fft_output.len() > 0);

        // Frequency resolution of the FFT (a.k.a. the frequency bin size)
        // is the sampling rate divided by the FFT buffer size.
        let freq_res = self.sampling_freq / fft_output.len() as f32;

        // Using the same unit circle analogy found here: https://dsp.stackexchange.com/q/2970/43899
        // The zero index is skipped, since the zero frequency does not apply here.
        let valid_fft_indices = 1..=(fft_output.len() / 2);

        let mut assignments = vec![(0.0f32, 0); self.spectrum.num_bands()];

        for i in valid_fft_indices {
            let freq_bin = freq_res * i as f32;

            // Where does this frequency bin fall in the band spectrum?
            if let Some(band_index) = self.spectrum.locate(freq_bin) {
                if let Some((value, count)) = assignments.get_mut(band_index) {
                    // println!("(i, f_bin, b_idx) = ({}, {}, {})", i, freq_bin, band_index);
                    *value += fft_output[i].norm();
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

        assert_eq!(self.spectrum.num_bands(), band_values.len());

        let total_sum = (&band_values).into_iter().sum::<f32>();

        if total_sum > 0.0 { band_values.into_iter().map(|x| x / total_sum).collect() }
        else { band_values }
    }
}

#[cfg(test)]
mod tests {
    use super::Assigner;

    use hound::WavReader;
    use rustfft::FFTplanner;
    use rustfft::num_complex::Complex;

    use crate::Error;

    use assert_approx_eq::assert_approx_eq;

    const PATH: &'static str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/wav/sin_440hz_44100hz_samp.wav",
    );

    fn get_fft() -> Vec<Complex<f32>> {
        let mut reader = WavReader::open(PATH).unwrap();
        let num_samples = reader.len() as usize;

        let mut planner = FFTplanner::new(false);
        let fft = planner.plan_fft(num_samples);

        let mut signal = reader.samples::<i32>()
            .map(|x| Complex::from(x.unwrap() as f32))
            .collect::<Vec<_>>();

        let mut spectrum = signal.clone();
        fft.process(&mut signal[..], &mut spectrum[..]);

        spectrum
    }

    #[test]
    fn test_assign_fft() {
        let fft_output = get_fft();

        let assigner = Assigner::new(20.0, 10000.0, 16, 44100.0).unwrap();

        let band_values = assigner.assign_fft(&fft_output);

        assert_approx_eq!(1.0f32, (&band_values).into_iter().sum::<f32>());
    }
}
