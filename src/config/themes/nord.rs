//! Nord theme - arctic, bluish dark theme.
//!
//! Reference: https://www.nordtheme.com

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Nord theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #2E3440 - Dark blue-gray background (polar night)
        background: rgb(46, 52, 64),
        // #3B4252 - Slightly lighter surface
        surface: rgb(59, 66, 82),
        // #ECEFF4 - Primary text (snow storm)
        text: rgb(236, 239, 244),
        // #4C566A - Muted secondary text (polar night lighter)
        dimmed: rgb(76, 86, 106),
        // #88C0D0 - Cyan/ice accent (frost)
        accent: rgb(136, 192, 208),
    }
}
