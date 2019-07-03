use rustfft::num_complex::Complex;

use crate::Error;
use crate::partition::BandPartitions;

pub struct Assigner {
    partitions: BandPartitions,
    sampling_freq: f32,
}

impl Assigner {
    pub fn new(lower_cutoff_freq: f32, upper_cutoff_freq: f32, num_bands: u16, sampling_freq: f32) -> Result<Self, Error> {
        if !(sampling_freq > 0.0) { Err(Error::InvalidSamplingRate)? }

        let upper_cutoff_freq = upper_cutoff_freq.min(sampling_freq / 2.0);

        let partitions = BandPartitions::new(lower_cutoff_freq, upper_cutoff_freq, num_bands)?;

        Ok(Self{ partitions, sampling_freq, })
    }

    pub fn assign_fft(&self, fft_output: &[Complex<f32>]) -> Vec<f32> {
        assert!(fft_output.len() > 0);

        // Frequency resolution of the FFT (a.k.a. the frequency bin size)
        // is the sampling rate divided by the FFT buffer size.
        let freq_res = self.sampling_freq / fft_output.len() as f32;

        // Using the same unit circle analogy found here: https://dsp.stackexchange.com/q/2970/43899
        // The zero index is skipped, since the zero frequency does not apply here.
        let valid_fft_indices = 1..=(fft_output.len() / 2);

        let mut assignments = vec![(0.0f32, 0); self.partitions.num_bands()];

        for i in valid_fft_indices {
            let freq_bin = freq_res * i as f32;

            // Where does this frequency bin fall in the band partitions?
            if let Some(band_index) = self.partitions.locate(freq_bin) {
                if let Some((value, count)) = assignments.get_mut(band_index) {
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

        assert_eq!(self.partitions.num_bands(), band_values.len());

        band_values
    }
}
