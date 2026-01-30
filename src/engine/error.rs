use std::fmt;
use std::io;

pub enum SpeedyError {
    IoError(io::Error),
    EmptyFile(String),
}

impl fmt::Display for SpeedyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeedyError::IoError(err) => write!(f, "I/O error: {}", err),
            SpeedyError::EmptyFile(path) => write!(f, "File is empty: {}", path),
        }
    }
}

impl fmt::Debug for SpeedyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for SpeedyError {}

impl From<io::Error> for SpeedyError {
    fn from(err: io::Error) -> Self {
        SpeedyError::IoError(err)
    }
}
