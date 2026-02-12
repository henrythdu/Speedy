use ratatui::style::Color;

/// Midnight theme colors (PRD Section 4.1)
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::midnight()
    }
}

impl Theme {
    /// Midnight theme (PRD Section 4.1)
    pub fn midnight() -> Self {
        Self {
            background: Color::Rgb(26, 27, 38), // #1A1B26 Stormy Dark
            surface: Color::Rgb(36, 40, 59),    // #24283B Dark Slate (command deck)
            text: Color::Rgb(169, 177, 214),    // #A9B1D6 Light Blue
            accent: Color::Rgb(247, 118, 142),  // #F7768E Coral Red
        }
    }

    /// Default theme is midnight
    pub fn current() -> Self {
        Self::midnight()
    }
}

/// Convenience access to current theme colors
pub mod colors {
    use super::Theme;
    use ratatui::style::Color;

    pub fn text() -> Color {
        Theme::current().text
    }
    pub fn accent() -> Color {
        Theme::current().accent
    }
    pub fn surface() -> Color {
        Theme::current().surface
    }
}
