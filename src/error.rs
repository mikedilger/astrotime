use std::error::Error as StdError;
use std::fmt;

/// Error type for the crate
#[derive(Debug)]
pub enum Error {
    /// General errors
    General(String),
    /// Out of Range
    RangeError,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::General(ref s) => write!(f, "{s}"),
            Self::RangeError => write!(f, "Value provided is out of range"),
        }
    }
}
