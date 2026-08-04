use ratatui::style::Color;
use std::cell::RefCell;

thread_local! {
    static CURRENT_THEME_NAME: RefCell<String> = RefCell::new(String::from("midnight"));
}

/// Set the current theme name (called from terminal.rs draw loop)
pub fn set_current_theme(name: &str) {
    CURRENT_THEME_NAME.with(|n| {
        *n.borrow_mut() = name.to_string();
    });
}

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
        Theme::tokyo_night()
    }
}

impl Theme {
    /// Tokyo Night theme (the default — PRD Section 4.1)
    pub fn tokyo_night() -> Self {
        Self {
            background: Color::Rgb(26, 27, 38), // #1A1B26 Stormy Dark
            surface: Color::Rgb(36, 40, 59),    // #24283B Dark Slate (command deck)
            text: Color::Rgb(169, 177, 214),    // #A9B1D6 Light Blue
            accent: Color::Rgb(247, 118, 142),  // #F7768E Coral Red
        }
    }

    /// Dracula theme - popular dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            background: Color::Rgb(40, 42, 54), // #282A36
            surface: Color::Rgb(68, 71, 90),    // #44475A
            text: Color::Rgb(248, 248, 242),    // #F8F8F2
            accent: Color::Rgb(255, 121, 198),  // #FF79C6
        }
    }

    /// Gruvbox theme - retro groove colors
    pub fn gruvbox() -> Self {
        Self {
            background: Color::Rgb(40, 40, 40), // #282828
            surface: Color::Rgb(60, 56, 54),    // #3C3834
            text: Color::Rgb(235, 219, 178),    // #EBDBB2
            accent: Color::Rgb(251, 73, 52),    // #FB4934
        }
    }

    /// Catppuccin Mocha theme - soothing pastel colors
    pub fn catppuccin_mocha() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46), // #1E1E2E
            surface: Color::Rgb(49, 50, 68),    // #313244
            text: Color::Rgb(205, 214, 244),    // #CDD6F4
            accent: Color::Rgb(243, 139, 168),  // #F38BA8
        }
    }

    /// Nord theme - arctic, bluish color palette
    pub fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64), // #2E3440
            surface: Color::Rgb(59, 66, 82),    // #3B4252
            text: Color::Rgb(236, 239, 244),    // #ECEFF4
            accent: Color::Rgb(136, 192, 208),  // #88C0D0
        }
    }

    /// Light theme - for bright environments
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(250, 250, 250), // #FAFAFA
            surface: Color::Rgb(240, 240, 240),    // #F0F0F0
            text: Color::Rgb(60, 60, 60),          // #3C3C3C
            accent: Color::Rgb(200, 80, 80),       // #C85050
        }
    }

    /// Get theme by name. Falls back to midnight for unknown names.
    pub fn get_by_name(name: &str) -> Self {
        match name {
            "tokyo-night" => Self::tokyo_night(),
            "dracula" => Self::dracula(),
            "gruvbox" => Self::gruvbox(),
            "catppuccin-mocha" | "catppuccin" => Self::catppuccin_mocha(),
            "nord" => Self::nord(),
            "light" => Self::light(),
            _ => Self::tokyo_night(),
        }
    }

    /// Get current theme from thread-local storage
    pub fn current() -> Self {
        CURRENT_THEME_NAME.with(|name| Self::get_by_name(&name.borrow()))
    }
}

/// Convenience access to current theme colors
pub mod colors {
    use crate::ui::theme::Theme;
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
