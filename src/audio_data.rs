use crate::Error;
use crate::sample::SampleBuffer;
use crate::types::SignalStrength;

pub enum AudioBuffer {
    Mono(SampleBuffer),
    Stereo(SampleBuffer, SampleBuffer),
}

impl AudioBuffer {
    pub fn len(&self) -> usize {
        match self {
            &AudioBuffer::Mono(ref buf) => buf.len(),
            &AudioBuffer::Stereo(ref buf_l, _) => buf_l.len(),
        }
    }

    pub fn rms(&self, num_samples: usize) -> Result<SignalStrength, Error> {
        if num_samples == 0 { return Ok(0.0) }
        if self.len() < num_samples { Err(Error::NotEnoughSamples)? }

        let skip_amount = self.len() - num_samples;

        // Taken from http://replaygain.hydrogenaud.io/proposal/rms_energy.html
        let sum: SignalStrength = match self {
            &AudioBuffer::Mono(ref buf) => {
                buf.iter_tail().take(num_samples).map(|v| v.powi(2)).sum()

                // buf.iter().skip(skip_amount).copied().map(|v| v.powi(2)).sum()
            },
            &AudioBuffer::Stereo(ref buf_l, ref buf_r) => {
                let iter_l = buf_l.iter().skip(skip_amount).copied();
                let iter_r = buf_r.iter().skip(skip_amount).copied();
                iter_l.zip(iter_r).map(|(vl, vr)| ((vl + vr) / 2.0).powi(2)).sum()
            },
        };

        let rms_vol = (sum / num_samples as SignalStrength).sqrt();

        Ok(rms_vol)
    }
}
