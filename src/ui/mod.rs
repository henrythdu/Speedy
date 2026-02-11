pub mod autocomplete;
pub mod command;
pub mod command_executor;
pub mod reader;
pub mod terminal;
pub mod theme;

pub use terminal::TuiManager;

use thiserror::Error;

/// UI-related errors
#[derive(Error, Debug, Clone)]
pub enum UIError {
    /// Lock poisoned error
    /// TODO: Will be used when thread safety is implemented for state sharing
    #[allow(dead_code)]
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}

impl UIError {
    /// Create a new LockPoisoned error
    /// TODO: Will be used when thread safety is implemented for state sharing
    #[allow(dead_code)]
    pub fn lock_poisoned(name: impl Into<String>) -> Self {
        Self::LockPoisoned(name.into())
    }
}
