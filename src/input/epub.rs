use super::{LoadError, LoadedDocument};

pub fn load(path: &str) -> Result<LoadedDocument, LoadError> {
    // TODO: Implement EPUB loading (Task 2A-4, Speedy-1r3)
    Err(LoadError::EpubParse("Not yet implemented".to_string()))
}
