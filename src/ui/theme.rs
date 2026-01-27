use ratatui::style::Color;

/// Midnight theme colors (PRD Section 4.1)
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub anchor: Color,
    pub dimmed: Color,
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
            text: Color::Rgb(169, 177, 214),    // #A9B1D6 Light Blue
            anchor: Color::Rgb(247, 118, 142),  // #F7768E Coral Red
            dimmed: Color::Rgb(100, 110, 150),  // #646E96 Dimmed Blue
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

    pub fn background() -> Color {
        Theme::current().background
    }
    pub fn text() -> Color {
        Theme::current().text
    }
    pub fn anchor() -> Color {
        Theme::current().anchor
    }
    pub fn dimmed() -> Color {
        Theme::current().dimmed
    }
}
