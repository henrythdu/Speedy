use std::fmt;
use std::io;

pub enum SpeedyError {
    IoError(io::Error),
    EmptyFile(String),
    InvalidEncoding(String),
}

impl fmt::Display for SpeedyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeedyError::IoError(err) => write!(f, "I/O error: {}", err),
            SpeedyError::EmptyFile(path) => write!(f, "File is empty: {}", path),
            SpeedyError::InvalidEncoding(path) => write!(f, "Invalid file encoding: {}", path),
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

pub fn load_file_safe(path: &str) -> Result<String, SpeedyError> {
    let content = std::fs::read_to_string(path)?;
    
    if content.trim().is_empty() {
        return Err(SpeedyError::EmptyFile(path.to_string()));
    }
    
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_empty_file_error() {
        let test_file = "test_empty.txt";
        File::create(test_file).unwrap();
        
        let result = load_file_safe(test_file);
        assert!(result.is_err());
        match result {
            Err(SpeedyError::EmptyFile(_)) => (),
            _ => panic!("Expected EmptyFile error"),
        }
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_nonexistent_file_error() {
        let result = load_file_safe("nonexistent_file_12345.txt");
        assert!(result.is_err());
        match result {
            Err(SpeedyError::IoError(_)) => (),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_valid_file_loads() {
        let test_file = "test_valid.txt";
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"hello world").unwrap();
        
        let result = load_file_safe(test_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
        
        fs::remove_file(test_file).unwrap();
    }
}
