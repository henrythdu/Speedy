pub mod config;
pub mod error;

pub mod capability;
pub mod cell_renderer;
pub mod font;
pub mod ovp;
pub mod renderer;
pub mod state;
pub mod timing;
pub mod viewport;

// Re-export reading module items to maintain backwards compatibility
pub use crate::reading::{
    calculate_anchor_position, tokenize_text, wpm_to_milliseconds, ReadingState, Token,
};
