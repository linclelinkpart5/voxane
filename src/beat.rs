use crate::Error;
use crate::types::SignalStrength;
use crate::types::Frequency;
use crate::analyzer::Analyzer;
use crate::window_kind::WindowKind;

pub struct Detector {
    decay: SignalStrength,
    trigger: SignalStrength,
    lower_cutoff: Frequency,
    upper_cutoff: Frequency,

    volume: SignalStrength,
    delta: SignalStrength,
    beat_delta: SignalStrength,
    peak: SignalStrength,
    valley: SignalStrength,

    analyzer: Analyzer,
}

impl Detector {
    pub fn new(
        decay_factor: usize,
        trigger: usize,
        trigger_factor: usize,
        lower_cutoff: usize,
        upper_cutoff: usize,
        fft_len: usize,
        ) -> Result<(), Error>
    {
        if !(decay_factor > 0) { Err(Error::DecayFactor)? }
        if !(trigger_factor > 0) { Err(Error::TriggerFactor)? }
        if !(lower_cutoff > 0) { Err(Error::LowerCutoff)? }
        if !(upper_cutoff > 0) { Err(Error::UpperCutoff)? }
        if !(lower_cutoff < upper_cutoff) { Err(Error::CutoffOrder)? }

        let decay = 1.0 - 1.0 / decay_factor as SignalStrength;
        let trigger = trigger as SignalStrength / trigger_factor as SignalStrength;

        let lower_cutoff = lower_cutoff as Frequency;
        let upper_cutoff = upper_cutoff as Frequency;

        let volume = 0.0;
        let delta = 0.0;
        let beat_delta = 0.0;
        let peak = 0.0;
        let valley = 0.0;

        let analyzer = Analyzer::new(fft_len, WindowKind::Blackman);

        Ok(())
    }
}
