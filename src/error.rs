/// An error that can occur when constructing a search index.
#[derive(Debug)]
pub enum Error {
    /// The provided text is invalid.
    InvalidText(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidText(msg) => write!(f, "invalid text: {}", msg,),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
