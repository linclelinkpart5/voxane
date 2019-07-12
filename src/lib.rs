#[cfg(test)] #[macro_use] extern crate assert_approx_eq;

pub mod assign;
pub mod audio;
pub mod types;
pub mod buckets;
pub mod analyzer;
pub mod window_kind;
pub mod wave;
pub mod sample;
pub mod fft_engine;
#[cfg(test)] pub mod test_util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNumBands,
    InvalidLowerCutoff,
    InvalidUpperCutoff,
    OutOfOrderCutoffs,
    InvalidSamplingRate,
    NotEnoughSamples,
    UnexpectedInputBufferSize(usize, usize),
    UnexpectedOutputBufferSize(usize, usize),
    NumSamples(usize, usize),
    SamplingRate(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidNumBands => write!(f, "number of bands must be greater than zero"),
            Error::InvalidLowerCutoff => write!(f, "lower cutoff must be greater than zero"),
            Error::InvalidUpperCutoff => write!(f, "upper cutoff must be greater than zero"),
            Error::OutOfOrderCutoffs => write!(f, "lower cutoff must be less than upper cutoff"),
            Error::InvalidSamplingRate => write!(f, "sampling rate must be greater than zero"),
            Error::NotEnoughSamples => write!(f, "not enough samples to fill buffer"),
            Error::UnexpectedInputBufferSize(e, p) => write!(f, "unexpected input buffer size {{ expected: {}, produced: {} }}", e, p),
            Error::UnexpectedOutputBufferSize(e, p) => write!(f, "unexpected output buffer size {{ expected: {}, produced: {} }}", e, p),
            Error::NumSamples(e, p) => write!(f, "unexpected number of samples {{ expected: {}, produced: {} }}", e, p),
            Error::SamplingRate(s) => write!(f, "sampling rate must be greater than zero and finite {{ found: {} }}", s),
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
