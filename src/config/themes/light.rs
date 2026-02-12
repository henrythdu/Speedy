//! Light theme - daytime light mode for bright environments.
//!
//! Optimized for readability in well-lit conditions.

#![allow(dead_code)]

use crate::config::theme::{rgb, ThemeColors};

/// Returns the Light theme colors.
pub fn colors() -> ThemeColors {
    ThemeColors {
        // #FBFBFB - Nearly white background
        background: rgb(251, 251, 251),
        // #E5E5E5 - Slightly darker surface for contrast
        surface: rgb(229, 229, 229),
        // #383A42 - Primary text (dark gray)
        text: rgb(56, 58, 66),
        // #A0A1A7 - Muted secondary text (medium gray)
        dimmed: rgb(160, 161, 167),
        // #E45649 - Red accent (one-dark inspired)
        accent: rgb(228, 86, 73),
    }
}
