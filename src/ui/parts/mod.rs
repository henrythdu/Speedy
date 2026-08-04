//! UI parts — one module per visible element.
//!
//! Each part owns its full rendering lifecycle: kitty images, ratatui cells,
//! position/anchor math, cache state and tests. A bug in one part (bar jitter,
//! deck bg, word flicker) has exactly one file to look in.
//!
//! - `word.rs` — current word + ghost words (the RSVP display itself)
//! - `progress.rs` — sentence-progress bar + document-progress gutter
//! - `background.rs` — rounded background card (kitty image + cell fill)
//! - `deck.rs` — command deck + WPM label

pub mod background;
pub mod deck;
pub mod help;
pub mod progress;
pub mod word;
