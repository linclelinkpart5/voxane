pub mod partition;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    InvalidNumBands,
    InvalidBaseFrequency,
    InvalidSamplingFrequency,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidNumBands => write!(f, "number of bands must be greater than zero"),
            Error::InvalidBaseFrequency => write!(f, "base frequency must be greater than zero and less than half of sampling frequency"),
            Error::InvalidSamplingFrequency => write!(f, "sampling frequency must be greater than zero"),
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
