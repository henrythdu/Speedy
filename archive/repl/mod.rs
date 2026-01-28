//! REPL (Read-Eval-Print Loop) module
//!
//! Provides REPL input, parsing, and rustyline integration.
//!
//! ## Module Structure
//!
//! - **command.rs**: Command definitions and conversion to AppEvent
//! - **parser.rs**: Manual string parsing for `@` and `:` prefixes
//! - **input.rs**: Rustyline wrapper with Helper trait for file completion and history
//!
//! ## Usage in main.rs
//!
//! ```rust,ignore
//! use speedy::repl::{input, parser};
//!
//! loop {
//!     let mut repl = input::ReplInput::new()?;
//!     let line = repl.readline()?;
//!     let event = parser::parse_repl_input(&line);
//!     app.handle_event(event);
//! }
//! ```

pub mod command;
pub mod input;
pub mod parser;

// Re-export public types
pub use command::ReplCommand;
pub use input::ReplInput;
