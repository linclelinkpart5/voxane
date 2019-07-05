pub mod partition;
pub mod assign;
pub mod audio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNumBands,
    InvalidLowerCutoff,
    InvalidUpperCutoff,
    OutOfOrderCutoffs,
    InvalidSamplingRate,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidNumBands => write!(f, "number of bands must be greater than zero"),
            Error::InvalidLowerCutoff => write!(f, "lower cutoff must be greater than zero"),
            Error::InvalidUpperCutoff => write!(f, "upper cutoff must be greater than zero"),
            Error::OutOfOrderCutoffs => write!(f, "lower cutoff must be less than upper cutoff"),
            Error::InvalidSamplingRate => write!(f, "sampling rate must be greater than zero"),
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
