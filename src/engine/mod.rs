pub mod config;

// Re-export reading module items to maintain backwards compatibility
pub use crate::reading::{
    tokenize_text, wpm_to_milliseconds, ReadingState, Token,
};
