//! Catppuccin Mocha theme - pastel, modern dark theme.
//!
//! Reference: https://github.com/catppuccin/catppuccin

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Catppuccin Mocha theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #1E1E2E - Dark blue-tinted background
        background: rgb(30, 30, 46),
        // #313244 - Slightly lighter surface (surface0)
        surface: rgb(49, 50, 68),
        // #CDD6F4 - Primary text (lavender-white)
        text: rgb(205, 214, 244),
        // #6C7086 - Muted secondary text (overlay0)
        dimmed: rgb(108, 112, 134),
        // #F38BA8 - Pink/red accent (maroon)
        accent: rgb(243, 139, 168),
    }
}
