use std::f32::consts::PI;

use crate::sample::Sample;
use crate::types::Frequency;

const AMPLITUDE: f32 = 0.25;

#[derive(Clone, Copy)]
pub enum WaveFunction {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    SineMag,
}

impl WaveFunction {
    pub fn val(&self, sample_index: usize, samples_per_period: usize, frequency: Frequency) -> Sample {
        let f_x = sample_index as f32 * frequency / samples_per_period as f32;
        AMPLITUDE * match self {
            &WaveFunction::Sine => (2.0 * PI * f_x).sin(),
            &WaveFunction::Square => (-1.0f32).powf((2.0 * f_x).floor()),
            &WaveFunction::Triangle => 1.0 - 4.0 * (0.5 - (f_x + 0.25).fract()).abs(),
            &WaveFunction::Sawtooth => 2.0 * f_x.fract() - 1.0,
            &WaveFunction::SineMag => 2.0 * (PI * f_x).sin().abs() - 1.0,
        }
    }
}

pub struct WaveGen {
    function: WaveFunction,
    samples_per_period: usize,
    sample_index: usize,
    frequency: Frequency,
}

impl WaveGen {
    pub fn new(function: WaveFunction, samples_per_period: usize, frequency: Frequency) -> Self {
        Self {
            function,
            samples_per_period,
            sample_index: 0,
            frequency,
        }
    }
}

impl Iterator for WaveGen {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.function.val(self.sample_index, self.samples_per_period, self.frequency);
        self.sample_index = (self.sample_index + 1) % self.samples_per_period;
        Some(v)
    }
}
