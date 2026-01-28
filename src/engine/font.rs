use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use lazy_static::lazy_static;

const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.otf");

lazy_static! {
    static ref EMBEDDED_FONT: Option<FontRef<'static>> = {
        FontRef::try_from_slice(JETBRAINS_MONO_BYTES).ok()
    };
}

pub fn get_font() -> Option<FontRef<'static>> {
    EMBEDDED_FONT.clone()
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
        assert!(metrics.line_gap() >= 0.0, "Font should have non-negative line gap");
    }
}
