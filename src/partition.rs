use super::Error;

pub struct BandPartitions(Vec<(f32, f32)>);

impl BandPartitions {
    // Inspired by https://stackoverflow.com/a/10462090/388739
    pub fn new(sampling_freq: f32, base_freq: f32, num_bands: u16) -> Result<Self, Error> {
        if num_bands == 0 { Err(Error::InvalidNumBands)? }
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
}
