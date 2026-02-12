//! Dracula theme - popular dark theme with vibrant colors.
//!
//! Reference: https://draculatheme.com

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Dracula theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #282A36 - Dark purple-tinted background
        background: rgb(40, 42, 54),
        // #44475A - Slightly lighter surface (current line)
        surface: rgb(68, 71, 90),
        // #F8F8F2 - Primary text (off-white)
        text: rgb(248, 248, 242),
        // #6272A4 - Muted secondary text (comment)
        dimmed: rgb(98, 114, 164),
        // #FF79C6 - Pink accent
        accent: rgb(255, 121, 198),
    }
}
