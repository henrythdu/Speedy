//! Kitty Graphics Protocol leaf utilities.
//!
//! Pure infrastructure shared by the UI parts (word, progress, background):
//! APC transmission, glyph rasterization, anchor/positioning math. The parts
//! themselves live in `src/ui/parts/`.

pub mod positioning;
pub mod protocol;
pub mod rasterizer;
