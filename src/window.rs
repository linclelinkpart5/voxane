use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub enum Window {
    Blackman,
    Hamming,
    Hanning,
    Rectangular,
    Sine,
    Triangular,
}

impl Window {
    pub fn generate(&self, len: usize) -> Vec<f64> {
        match self {
            &Window::Blackman => apodize::blackman_iter(len).collect(),
            &Window::Hamming => apodize::hamming_iter(len).collect(),
            &Window::Hanning => apodize::hanning_iter(len).collect(),
            &Window::Rectangular => vec![1.0; len],
            &Window::Sine => (0..len).map(|i| (i as f64 / (len - 1) as f64 * PI).sin()).collect(),
            &Window::Triangular => apodize::triangular_iter(len).collect(),
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Window::Rectangular
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_generate() {
        const LEN: usize = 8;

        let inputs_and_expected = vec![
            (Window::Blackman, vec![0.000060000000000004494, 0.03339172347815117, 0.332833504298565, 0.8893697722232837, 0.889369772223284, 0.3328335042985651, 0.03339172347815122, 0.000060000000000004494]),
            (Window::Hamming, vec![0.08000000000000002, 0.25319469114498255, 0.6423596296199047, 0.9544456792351128, 0.9544456792351128, 0.6423596296199048, 0.25319469114498266, 0.08000000000000002]),
            (Window::Hanning, vec![0.0, 0.1882550990706332, 0.6112604669781572, 0.9504844339512095, 0.9504844339512095, 0.6112604669781573, 0.1882550990706333, 0.0]),
            (Window::Rectangular, vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            (Window::Sine, vec![0.0, 0.4338837391175581, 0.7818314824680298, 0.9749279121818236, 0.9749279121818236, 0.7818314824680299, 0.43388373911755823, 0.00000000000000012246467991473532]),
            (Window::Triangular, vec![0.125, 0.375, 0.625, 0.875, 0.875, 0.625, 0.375, 0.125]),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.generate(LEN);

            for (e, p) in expected.into_iter().zip(produced) {
                assert_approx_eq!(e, p);
            }
        }
    }
}
