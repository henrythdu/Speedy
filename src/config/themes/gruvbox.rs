//! Gruvbox theme - warm, retro feel with earthy tones.
//!
//! Reference: https://github.com/morhetz/gruvbox

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Gruvbox theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #282828 - Dark background
        background: rgb(40, 40, 40),
        // #3C3836 - Slightly lighter surface
        surface: rgb(60, 56, 54),
        // #EBDBB2 - Primary text (creamy beige)
        text: rgb(235, 219, 178),
        // #928374 - Muted secondary text (gray)
        dimmed: rgb(146, 131, 116),
        // #FE8019 - Orange accent
        accent: rgb(254, 128, 25),
    }
}
