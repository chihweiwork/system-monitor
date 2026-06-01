use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    IoError(std::io::Error),  // Alias for compatibility with GPU module
    ParseError(String),
    CollectionError(String),
    UnsupportedPlatform,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) | Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::CollectionError(msg) => write!(f, "Collection error: {}", msg),
            Error::UnsupportedPlatform => write!(f, "Unsupported platform"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
