pub mod partition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidNumBands,
    InvalidLowerCutoff,
    InvalidUpperCutoff,
    InvalidSamplingRate,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidNumBands => write!(f, "number of bands must be greater than zero"),
            Error::InvalidLowerCutoff => write!(f, "lower cutoff must be greater than zero and less than upper cutoff"),
            Error::InvalidUpperCutoff => write!(f, "lower cutoff must be greater than zero and greater than lower cutoff"),
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
