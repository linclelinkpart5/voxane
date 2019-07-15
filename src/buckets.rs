use std::cmp::Ordering;

use crate::Error;
use crate::types::Frequency;
use crate::types::SignalStrength;

#[derive(Clone)]
pub struct Buckets(Vec<(Frequency, Frequency)>);

impl Buckets {
    // Inspired by https://stackoverflow.com/a/10462090/388739
    pub fn new(lower_cutoff: Frequency, upper_cutoff: Frequency, num_bands: usize) -> Result<Self, Error> {
        // Check invariants.
        if !(upper_cutoff > 0.0) { Err(Error::UpperCutoff)? }
        if !(lower_cutoff > 0.0) { Err(Error::LowerCutoff)? }
        if !(lower_cutoff < upper_cutoff) { Err(Error::CutoffOrder)? }

        // Space out logarithmically.
        let octave_factor = num_bands as f32 / (upper_cutoff / lower_cutoff).log2();
        let exp = 1.0 / octave_factor;
        let factor = 2.0f32.powf(exp);

        let mut partitions = Vec::with_capacity(num_bands as usize);

        let mut curr_lower_limit = lower_cutoff;

        for i in 1..=num_bands {
            let curr_upper_limit =
                if i == num_bands && upper_cutoff > curr_lower_limit { upper_cutoff }
                else { curr_lower_limit * factor }
            ;

            partitions.push((curr_lower_limit, curr_upper_limit));

            curr_lower_limit = curr_upper_limit;
        }

        Ok(Self(partitions))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn locate(&self, target: Frequency) -> Option<usize> {
        self.0.binary_search_by(|(lo, hi)| {
            match (lo <= &target, &target < hi) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => unreachable!("invalid/out-of-order band partition created"),
            }
        }).ok()
    }

    #[inline]
    pub fn bands(&self) -> &[(Frequency, Frequency)] {
        self.0.as_slice()
    }

    pub fn bucketize(&self, spectrum: &[SignalStrength], sampling_rate: usize) -> Result<Vec<SignalStrength>, Error> {
        if !(sampling_rate > 0) { Err(Error::SamplingRate(sampling_rate))? }

        let mut bucketized = vec![0.0f32; self.len()];

        match spectrum.len() {
            0 => {},
            n => {
                // Using the same unit circle analogy found here: https://dsp.stackexchange.com/q/2970/43899
                // The zero index is skipped, since the zero frequency does not apply here.
                let valid_fft_indices = 1..=(n / 2);

                let fft_bin_size = sampling_rate as f32 / n as f32;

                for i in valid_fft_indices {
                    let freq_bin = fft_bin_size * i as f32;

                    // Where does this frequency bin fall in the buckets?
                    if let Some(band_index) = self.locate(freq_bin) {
                        bucketized[band_index] += spectrum[i];
                    }
                }
            },
        };

        Ok(bucketized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil;

    #[test]
    fn test_new() {
        let expected: Vec<(Frequency, Frequency)> = vec![
            (10.0, 16.179424),
            (16.179424, 26.177376),
            (26.177376, 42.353485),
            (42.353485, 68.5255),
            (68.5255, 110.8703),
            (110.8703, 179.38176),
            (179.38176, 290.22934),
            (290.22934, 469.57434),
            (469.57434, 759.7442),
            (759.7442, 1229.2223),
            (1229.2223, 1988.8108),
            (1988.8108, 3217.7813),
            (3217.7813, 5206.1846),
            (5206.1846, 8423.307),
            (8423.307, 13628.425),
            (13628.425, 22050.0),
        ];
        let produced = Buckets::new(10.0, 22050.0, 16).unwrap();

        assert_eq!(expected.len(), produced.len());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let expected: Vec<(Frequency, Frequency)> = vec![
            (20.0, 48.520767),
            (48.520767, 117.71324),
            (117.71324, 285.57684),
            (285.57684, 692.8204),
            (692.8204, 1680.8087),
            (1680.8087, 4077.7063),
            (4077.7063, 9892.672),
            (9892.672, 24000.0),
        ];
        let produced = Buckets::new(20.0, 24000.0, 8).unwrap();

        assert_eq!(expected.len(), produced.len());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let produced = Buckets::new(20.0, 44100.0, 0).unwrap();
        assert_eq!(0, produced.len());
    }

    #[test]
    fn test_locate() {
        let partitions = Buckets::new(10.0, 22050.0, 16).unwrap();

        let inputs_and_expected = vec![
            (22049.9, Some(15)),
            (22050.0, None),
            (0.0, None),
            (9.9, None),
            (10.0, Some(0)),
            (500.0, Some(8)),
            (759.0, Some(8)),
            (760.0, Some(9)),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = partitions.locate(input);
            assert_eq!(expected, produced);
        }
    }

    const SAMPLES_PER_PERIOD: usize = 44100;
    const FREQUENCY: Frequency = 1000.0;

    #[test]
    fn test_bucketize() {
        use crate::analyzer::Analyzer;
        use crate::window_kind::WindowKind;

        let buckets = Buckets::new(20.0, 10000.0, 16).unwrap();

        let samples = TestUtil::generate_wave_samples(SAMPLES_PER_PERIOD, FREQUENCY, 1024);

        let mut analyzer = Analyzer::new(1024, WindowKind::default());

        let spectrum = analyzer.analyze(&samples).unwrap();

        let produced = buckets.bucketize(&spectrum, SAMPLES_PER_PERIOD).unwrap();

        let expected = vec![
            0.0,
            1.7029626,
            0.0,
            1.7407556,
            1.8054703,
            1.8997737,
            6.6312847,
            9.183684,
            30.809696,
            727.12177,
            15472.85,
            49.126057,
            17.683151,
            9.717427,
            6.2228317,
            4.263113,
        ];

        for (e, p) in expected.into_iter().zip(&produced) {
            assert_approx_eq!(e, p);
        }

        println!("{:?}", produced);
        println!("{:?}", buckets.bands());
    }
}

