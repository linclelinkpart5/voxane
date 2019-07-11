use crate::wave::WaveFunction;
use crate::wave::WaveGen;
use crate::types::Frequency;
use crate::sample::Sample;

pub struct TestUtil;

impl TestUtil {
    pub fn generate_wave_samples(samples_per_period: usize, frequency: Frequency, len: usize) -> Vec<Sample> {
        WaveGen::new(WaveFunction::Sine, samples_per_period, frequency).take(len).collect()
    }
}
