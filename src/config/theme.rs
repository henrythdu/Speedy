//! Theme color definitions for Speedy.
//!
//! Provides the `ThemeColors` struct with RGBA color fields for consistent
//! styling across the application.

#![allow(dead_code)]

/// RGBA color representation for theme elements.
///
/// Each color is stored as `[u8; 4]` with values `[R, G, B, A]` where:
/// - R, G, B: Red, Green, Blue components (0-255)
/// - A: Alpha/transparency (0-255, where 255 is fully opaque)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    /// Primary background color.
    pub background: [u8; 4],
    /// Secondary surface color (slightly lighter than background).
    pub surface: [u8; 4],
    /// Primary text color.
    pub text: [u8; 4],
    /// Secondary/muted text color.
    pub dimmed: [u8; 4],
    /// Accent/highlight color for emphasis.
    pub accent: [u8; 4],
}

/// Creates an RGBA color from RGB values with full opacity.
///
/// # Arguments
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)
/// * `b` - Blue component (0-255)
///
/// # Returns
/// Array `[r, g, b, 255]` representing the color with full alpha.
#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    [r, g, b, 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_helper() {
        assert_eq!(rgb(255, 128, 0), [255, 128, 0, 255]);
        assert_eq!(rgb(0, 0, 0), [0, 0, 0, 255]);
        assert_eq!(rgb(255, 255, 255), [255, 255, 255, 255]);
    }

    #[test]
    fn test_theme_colors_equality() {
        let theme1 = ThemeColors {
            background: rgb(26, 27, 38),
            surface: rgb(35, 37, 52),
            text: rgb(169, 177, 214),
            dimmed: rgb(90, 98, 128),
            accent: rgb(247, 118, 142),
        };
        let theme2 = ThemeColors {
            background: rgb(26, 27, 38),
            surface: rgb(35, 37, 52),
            text: rgb(169, 177, 214),
            dimmed: rgb(90, 98, 128),
            accent: rgb(247, 118, 142),
        };
        assert_eq!(theme1, theme2);
    }
}
