use std::cmp::Ordering;

use super::Error;

pub struct BandPartitions(Vec<(f32, f32)>);

impl BandPartitions {
    // Inspired by https://stackoverflow.com/a/10462090/388739
    pub fn new(sampling_freq: f32, base_freq: f32, num_bands: u16) -> Result<Self, Error> {
        if !(base_freq > 0.0) { Err(Error::InvalidBaseFrequency)? }
        if !(base_freq < sampling_freq / 2.0) { Err(Error::InvalidBaseFrequency)? }
        if !(sampling_freq > 0.0) { Err(Error::InvalidSamplingFrequency)? }

        let octave_factor = num_bands as f32 / ((sampling_freq / base_freq).log2() - 1.0);
        let exp = 1.0 / octave_factor;

        let factor = 2.0f32.powf(-exp);

        let mut partitions = Vec::with_capacity(num_bands as usize);

        let mut curr_upper_limit = sampling_freq / 2.0;

        for i in 1..=num_bands {
            let curr_lower_limit =
                if i == num_bands && base_freq < curr_upper_limit { base_freq }
                else { curr_upper_limit * factor }
            ;

            partitions.push((curr_lower_limit, curr_upper_limit));

            curr_upper_limit = curr_lower_limit;
        }

        partitions.reverse();

        Ok(Self(partitions))
    }

    pub fn num_bands(&self) -> usize {
        self.0.len()
    }

    pub fn locate(&self, target_freq: f32) -> Option<usize> {
        self.0.binary_search_by(|(lo, hi)| {
            match (lo <= &target_freq, &target_freq < hi) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => unreachable!("invalid band partition created"),
            }
        }).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::BandPartitions;

    use crate::Error;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() -> Result<(), Error> {
        let expected: Vec<(f32, f32)> = vec![
            (10.0, 16.179422),
            (16.179422, 26.177372),
            (26.177372, 42.353477),
            (42.353477, 68.52548),
            (68.52548, 110.87028),
            (110.87028, 179.38171),
            (179.38171, 290.22928),
            (290.22928, 469.57425),
            (469.57425, 759.7441),
            (759.7441, 1229.2222),
            (1229.2222, 1988.8105),
            (1988.8105, 3217.7808),
            (3217.7808, 5206.1836),
            (5206.1836, 8423.305),
            (8423.305, 13628.421),
            (13628.421, 22050.0),
        ];
        let produced = BandPartitions::new(44100.0, 10.0, 16)?;

        assert_eq!(expected.len(), produced.num_bands());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let expected: Vec<(f32, f32)> = vec![
            (20.0, 48.52076),
            (48.52076, 117.71322),
            (117.71322, 285.57678),
            (285.57678, 692.82025),
            (692.82025, 1680.8085),
            (1680.8085, 4077.7058),
            (4077.7058, 9892.671),
            (9892.671, 24000.0),
        ];
        let produced = BandPartitions::new(48000.0, 20.0, 8)?;

        assert_eq!(expected.len(), produced.num_bands());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let produced = BandPartitions::new(44100.0, 20.0, 0)?;
        assert_eq!(0, produced.num_bands());

        Ok(())
    }

    #[test]
    fn test_locate() -> Result<(), Error> {
        let partitions = BandPartitions::new(44100.0, 10.0, 16)?;

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

        Ok(())
    }
}
