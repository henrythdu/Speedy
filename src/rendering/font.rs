use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use lazy_static::lazy_static;

const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.otf");

lazy_static! {
    static ref EMBEDDED_FONT: Option<FontRef<'static>> =
        FontRef::try_from_slice(JETBRAINS_MONO_BYTES).ok();
}

pub fn get_font() -> Option<FontRef<'static>> {
    EMBEDDED_FONT.clone()
}

pub fn calculate_char_width(font: &FontRef, c: char, font_size: f32) -> f32 {
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);

    let glyph_id = font.glyph_id(c);
    scaled_font.h_advance(glyph_id)
}

pub fn calculate_string_width(font: &FontRef, text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|c| calculate_char_width(font, c, font_size))
        .sum()
}

pub fn get_font_metrics(font: &FontRef, font_size: f32) -> FontMetrics {
    let scale = PxScale::from(font_size);
    let metrics = font.as_scaled(scale);

    FontMetrics {
        ascent: metrics.ascent(),
        descent: metrics.descent(),
        line_gap: metrics.line_gap(),
        height: metrics.height(),
        font_size,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub height: f32,
    pub font_size: f32,
}

pub fn load_font_from_path<P: AsRef<std::path::Path>>(path: P) -> Option<FontRef<'static>> {
    std::fs::read(path).ok().and_then(|bytes| {
        // Leak the bytes to get 'static lifetime
        let leaked_bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        FontRef::try_from_slice(leaked_bytes).ok()
    })
}

pub struct FontConfig {
    pub custom_font_path: Option<String>,
    pub font_size: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            custom_font_path: None,
            font_size: 24.0,
        }
    }
}

pub fn get_font_with_config(config: &FontConfig) -> Option<FontRef<'static>> {
    config
        .custom_font_path
        .as_ref()
        .and_then(|path| load_font_from_path(path))
        .or_else(|| get_font())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loads_from_embedded_bytes() {
        let font = get_font();
        assert!(font.is_some(), "Font should load from embedded bytes");
    }

    #[test]
    fn test_font_provides_metrics() {
        let font = get_font().expect("Font should be available");
        let scale = PxScale::from(24.0);
        let metrics = font.as_scaled(scale);

        assert!(metrics.ascent() > 0.0, "Font should have positive ascent");
        assert!(metrics.descent() < 0.0, "Font should have negative descent");
        assert!(
            metrics.line_gap() >= 0.0,
            "Font should have non-negative line gap"
        );
    }

    #[test]
    fn test_calculate_character_width() {
        let font = get_font().expect("Font should be available");
        let width = calculate_char_width(&font, 'W', 24.0);

        assert!(width > 0.0, "Character width should be positive");
        // JetBrains Mono is a monospace font, so all characters have same width
        assert_eq!(
            width,
            calculate_char_width(&font, 'i', 24.0),
            "Monospace font: 'W' and 'i' should have same width"
        );
    }

    #[test]
    fn test_calculate_string_width() {
        let font = get_font().expect("Font should be available");
        let width = calculate_string_width(&font, "Hello", 24.0);

        assert!(width > 0.0, "String width should be positive");
    }

    #[test]
    fn test_load_font_from_path() {
        // This test uses the embedded font path for simplicity
        let font = load_font_from_path("assets/fonts/JetBrainsMono-Regular.otf");
        assert!(font.is_some(), "Should load font from file path");
    }

    #[test]
    fn test_load_font_from_invalid_path() {
        let font = load_font_from_path("/nonexistent/font.ttf");
        assert!(font.is_none(), "Should return None for invalid path");
    }
}
