use std::f32::consts::PI;

use crate::types::Sample;

const AMPLITUDE: f32 = 0.25;
const FREQUENCY: f32 = 440.0;

#[derive(Clone, Copy)]
pub enum WaveFunction {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    SineMag,
}

impl WaveFunction {
    pub fn val(&self, sample_clock: u32, sample_rate: u32) -> Sample {
        let f_x = sample_clock as f32 * FREQUENCY / sample_rate as f32;
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
    sample_rate: u32,
    sample_clock: u32,
}

impl WaveGen {
    pub fn new(function: WaveFunction, sample_rate: u32) -> Self {
        Self {
            function,
            sample_rate,
            sample_clock: 0,
        }
    }
}

impl Iterator for WaveGen {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.function.val(self.sample_clock, self.sample_rate);
        self.sample_clock = (self.sample_clock + 1) % self.sample_rate;
        Some(v)
    }
}
