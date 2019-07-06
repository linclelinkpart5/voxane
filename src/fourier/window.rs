use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub enum Window {
    Blackman,
    Hamming,
    Hanning,
    Rectangle,
    Nuttall,
    Sine,
    Triangular,
}

impl Window {
    pub fn generate(&self, len: usize) -> Vec<f64> {
        match self {
            &Window::Blackman => apodize::blackman_iter(len).collect(),
            &Window::Hamming => apodize::hamming_iter(len).collect(),
            &Window::Hanning => apodize::hanning_iter(len).collect(),
            &Window::Rectangle => vec![1.0; len],
            &Window::Nuttall => apodize::nuttall_iter(len).collect(),
            &Window::Sine => (0..len).map(|i| (i as f64 / (len - 1) as f64 * PI).sin()).collect(),
            &Window::Triangular => apodize::triangular_iter(len).collect(),
        }
    }
}
