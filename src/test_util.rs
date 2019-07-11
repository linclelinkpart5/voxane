use crate::wave::WaveFunction;
use crate::wave::WaveGen;
use crate::types::Frequency;
use crate::sample::Sample;

pub struct TestUtil;

impl TestUtil {
    pub fn generate_wave_samples(wave_function: WaveFunction, samples_per_period: usize, frequency: Frequency, len: usize) -> Vec<Sample> {
        WaveGen::new(wave_function, samples_per_period, frequency).take(len).collect()
    }
}
