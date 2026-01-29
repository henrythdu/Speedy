pub mod config;
pub mod error;

// Re-export reading module items to maintain backwards compatibility
pub use crate::reading::{
    calculate_anchor_position, tokenize_text, wpm_to_milliseconds, ReadingState, Token,
};
