use std::sync::Arc;

use rustfft::FFT;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;

use crate::Error;
use crate::sample::Sample;
use crate::types::SignalStrength;
use crate::window_kind::WindowKind;

pub struct FFTEngine {
    // Reusable FFT algorithm.
    fft: Arc<dyn FFT<Sample>>,

    // FFT window values to use for smoothing.
    window: Vec<f32>,
}

impl FFTEngine {
    pub fn new(len: usize, window_kind: WindowKind) -> Self {
        let fft = FFTplanner::new(false).plan_fft(len);
        let window = window_kind.generate(len).into_iter().map(|x| x as f32).collect();

        Self { fft, window, }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fft.len()
    }

    pub fn process(&self, input_buf: &mut [Complex<Sample>], output_buf: &mut [Complex<SignalStrength>]) -> Result<(), Error> {
        if self.len() != input_buf.len() { Err(Error::UnexpectedInputBufferSize(self.len(), input_buf.len()))? }
        if self.len() != output_buf.len() { Err(Error::UnexpectedOutputBufferSize(self.len(), output_buf.len()))? }

        for (i, w) in input_buf.iter_mut().zip(&self.window) {
            // TODO: Which is more correct?
            // (*i).re *= w;
            *i *= w;
        }

        self.fft.process(input_buf, output_buf);

        Ok(())
    }
}
