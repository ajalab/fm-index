//! Suffix arrays, used to construct the index.
//!
//! Can also be used in sampled fashion to perform locate queries.

pub mod sais;
pub mod sample;

/// An error that can occur when building a suffix array.
#[derive(Debug)]
pub(crate) enum Error {
    /// The given text cannot be used to build a suffix array.
    InvalidText(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidText(msg) => write!(f, "invalid text: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
