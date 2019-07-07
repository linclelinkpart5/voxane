use std::cmp::Ordering;

use crate::Error;
use crate::types::Frequency;

#[derive(Clone)]
pub struct Spectrum(Vec<(Frequency, Frequency)>);

impl Spectrum {
    // Inspired by https://stackoverflow.com/a/10462090/388739
    pub fn new(lower_cutoff: Frequency, upper_cutoff: Frequency, num_bands: usize) -> Result<Self, Error> {
        // Check invariants.
        if !(upper_cutoff > 0.0) { Err(Error::InvalidUpperCutoff)? }
        if !(lower_cutoff > 0.0) { Err(Error::InvalidLowerCutoff)? }
        if !(lower_cutoff < upper_cutoff) { Err(Error::OutOfOrderCutoffs)? }

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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn lower_cutoff(&self) -> Option<Frequency> {
        self.0.first().map(|(f, _)| *f)
    }

    pub fn upper_cutoff(&self) -> Option<Frequency> {
        self.0.last().map(|(_, f)| *f)
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
}

#[cfg(test)]
mod tests {
    use super::Spectrum;

    use crate::Error;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_new() -> Result<(), Error> {
        let expected: Vec<(f32, f32)> = vec![
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
        let produced = Spectrum::new(10.0, 22050.0, 16)?;

        assert_eq!(expected.len(), produced.len());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let expected: Vec<(f32, f32)> = vec![
            (20.0, 48.520767),
            (48.520767, 117.71324),
            (117.71324, 285.57684),
            (285.57684, 692.8204),
            (692.8204, 1680.8087),
            (1680.8087, 4077.7063),
            (4077.7063, 9892.672),
            (9892.672, 24000.0),
        ];
        let produced = Spectrum::new(20.0, 24000.0, 8)?;

        assert_eq!(expected.len(), produced.len());
        for (e, p) in expected.into_iter().zip(produced.0) {
            assert_approx_eq!(e.0, p.0);
            assert_approx_eq!(e.1, p.1);
        }

        let produced = Spectrum::new(20.0, 44100.0, 0)?;
        assert_eq!(0, produced.len());

        Ok(())
    }

    #[test]
    fn test_locate() -> Result<(), Error> {
        let partitions = Spectrum::new(10.0, 22050.0, 16)?;

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

