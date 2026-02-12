//! Tokyo Night theme - the default dark theme with blue-tinted colors.
//!
//! Reference: https://github.com/enkia/tokyo-night-vscode-theme

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Tokyo Night theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #1A1B26 - Dark blue-tinted background
        background: rgb(26, 27, 38),
        // #24283B - Slightly lighter surface
        surface: rgb(36, 40, 59),
        // #A9B1D6 - Primary text (soft blue-white)
        text: rgb(169, 177, 214),
        // #565F89 - Muted secondary text
        dimmed: rgb(86, 95, 137),
        // #F7768E - Red-pink accent
        accent: rgb(247, 118, 142),
    }
}
