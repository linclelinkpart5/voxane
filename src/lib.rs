#[cfg(test)] #[macro_use] extern crate assert_approx_eq;

pub mod types;
pub mod buckets;
pub mod analyzer;
pub mod window_kind;
pub mod wave;
pub mod sample;
pub mod listener;
pub mod beat;
#[cfg(test)] pub mod test_util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NumBands,
    LowerCutoff,
    UpperCutoff,
    CutoffOrder,
    InputBuffer(usize, usize),
    OutputBuffer(usize, usize),
    NumSamples(usize, usize),
    SamplingRate(usize),
    DecayFactor,
    TriggerFactor,
    TooFewSamples(usize, usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::NumBands => write!(f, "number of bands must be greater than zero"),
            Error::LowerCutoff => write!(f, "lower cutoff must be greater than zero"),
            Error::UpperCutoff => write!(f, "upper cutoff must be greater than zero"),
            Error::CutoffOrder => write!(f, "lower cutoff must be less than upper cutoff"),
            Error::InputBuffer(e, p) => write!(f, "unexpected input buffer size {{ expected: {}, produced: {} }}", e, p),
            Error::OutputBuffer(e, p) => write!(f, "unexpected output buffer size {{ expected: {}, produced: {} }}", e, p),
            Error::NumSamples(e, p) => write!(f, "unexpected number of samples {{ expected: {}, produced: {} }}", e, p),
            Error::SamplingRate(s) => write!(f, "sampling rate must be greater than zero and finite {{ found: {} }}", s),
            Error::DecayFactor => write!(f, "decay factor must be greater than zero"),
            Error::TriggerFactor => write!(f, "trigger factor must be greater than zero"),
            Error::TooFewSamples(e, p) => write!(f, "too few samples in buffer {{ expected: {}, produced: {} }}", e, p),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            _ => None,
        }
    }
}
