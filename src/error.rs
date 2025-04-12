/// An error that can occur when constructing a search index.
#[derive(Debug)]
pub enum Error {
    /// Failed to construct a suffix array from the given text.
    SuffixArray(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SuffixArray(msg) => write!(
                f,
                "failed to construct a suffix array from the given text: {}",
                msg,
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<crate::suffix_array::Error> for Error {
    fn from(err: crate::suffix_array::Error) -> Self {
        Error::SuffixArray(err.to_string())
    }
}
