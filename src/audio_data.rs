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
            &AudioBuffer::Stereo(ref buf_l, ref buf_r) => {
                assert_eq!(buf_l.len(), buf_r.len());
                buf_l.len()
            },
        }
    }

    pub fn root_mean_sqr(&self) -> SignalStrength {
        if self.len() == 0 { return 0.0 }

        // Taken from http://replaygain.hydrogenaud.io/proposal/rms_energy.html
        match self {
            &AudioBuffer::Mono(ref buf) => buf.root_mean_sqr(),
            &AudioBuffer::Stereo(ref buf_l, ref buf_r) => {
                let mean_sqr_l = buf_l.mean_sqr();
                let mean_sqr_r = buf_r.mean_sqr();

                ((mean_sqr_l + mean_sqr_r) / 2.0).sqrt()
            },
        }
    }
}
