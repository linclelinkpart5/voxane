pub mod assign;
pub mod audio;
pub mod types;
pub mod buckets;
pub mod analyzer;
pub mod window;
pub mod wave;
pub mod sample;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNumBands,
    InvalidLowerCutoff,
    InvalidUpperCutoff,
    OutOfOrderCutoffs,
    InvalidSamplingRate,
    NotEnoughSamples,
    IncorrectBufferSize(usize, usize),
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
            Error::IncorrectBufferSize(e, p) => write!(f, "incorrect buffer size {{ expected: {}, produced: {} }}", e, p),
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
