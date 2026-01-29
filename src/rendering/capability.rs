//! Terminal capability detection for graphics protocol support
//!
//! Detects terminal graphics capabilities and provides graceful fallback
//! to TUI mode when advanced features are unavailable.

use std::env;

/// Graphics protocol support levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicsCapability {
    /// No graphics support - use pure TUI fallback
    None,
    /// Kitty Graphics Protocol supported
    Kitty,
}

impl GraphicsCapability {
    /// Returns true if terminal supports pixel-perfect graphics
    pub fn supports_graphics(&self) -> bool {
        matches!(self, GraphicsCapability::Kitty)
    }

    /// Returns true if sub-pixel OVP positioning is supported
    pub fn supports_subpixel_ovp(&self) -> bool {
        matches!(self, GraphicsCapability::Kitty)
    }
}

/// Terminal capability detector
pub struct CapabilityDetector;

impl CapabilityDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self
    }

    /// Detect terminal graphics capability
    ///
    /// Currently checks environment variables ($TERM, $TERM_PROGRAM).
    ///
    /// TODO: Implement CSI DA1/DA2 device attribute queries as fallback
    /// for more reliable detection. This requires async/timeout handling.
    ///
    /// Returns GraphicsCapability::None for unsupported terminals
    pub fn detect(&self) -> GraphicsCapability {
        // First check environment variables (fast but less reliable)
        if let Some(capability) = self.detect_from_env() {
            return capability;
        }

        // TODO: Implement CSI query fallback (requires async/timeout handling)
        // For now, return None - CSI query implementation is pending
        GraphicsCapability::None
    }

    /// Detect capability from environment variables
    fn detect_from_env(&self) -> Option<GraphicsCapability> {
        // Check $TERM for known terminals
        if let Ok(term) = env::var("TERM") {
            let term_lower = term.to_lowercase();

            // Kitty terminal
            if term_lower.contains("kitty") || term_lower.contains("xterm-kitty") {
                return Some(GraphicsCapability::Kitty);
            }

            // Konsole (supports Kitty protocol)
            if term_lower.contains("konsole") {
                return Some(GraphicsCapability::Kitty);
            }
        }

        // Check $TERM_PROGRAM for macOS terminals
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            let program_lower = term_program.to_lowercase();

            if program_lower.contains("kitty") {
                return Some(GraphicsCapability::Kitty);
            }
        }

        None
    }

    /// Detect capability from explicit CLI override
    pub fn detect_from_override(
        &self,
        force_kitty: bool,
        force_tui: bool,
    ) -> Option<GraphicsCapability> {
        if force_kitty {
            return Some(GraphicsCapability::Kitty);
        }
        if force_tui {
            return Some(GraphicsCapability::None);
        }
        None
    }
}

impl Default for CapabilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Get user-facing warning message for TUI fallback mode
pub fn get_tui_fallback_warning() -> &'static str {
    "⚠️  Running in TUI fallback mode. For pixel-perfect RSVP with sub-pixel OVP anchoring, use Kitty or Konsole terminal."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_capability_variants() {
        assert!(!GraphicsCapability::None.supports_graphics());
        assert!(GraphicsCapability::Kitty.supports_graphics());

        assert!(!GraphicsCapability::None.supports_subpixel_ovp());
        assert!(GraphicsCapability::Kitty.supports_subpixel_ovp());
    }

    #[test]
    fn test_detector_creation() {
        let detector = CapabilityDetector::new();
        // Should not panic
        let _capability = detector.detect();
    }

    #[test]
    fn test_detect_from_override_force_kitty() {
        let detector = CapabilityDetector::new();
        let result = detector.detect_from_override(true, false);
        assert_eq!(result, Some(GraphicsCapability::Kitty));
    }

    #[test]
    fn test_detect_from_override_force_tui() {
        let detector = CapabilityDetector::new();
        let result = detector.detect_from_override(false, true);
        assert_eq!(result, Some(GraphicsCapability::None));
    }

    #[test]
    fn test_detect_from_override_no_override() {
        let detector = CapabilityDetector::new();
        let result = detector.detect_from_override(false, false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_detect_from_override_both_flags() {
        // force_kitty takes precedence
        let detector = CapabilityDetector::new();
        let result = detector.detect_from_override(true, true);
        assert_eq!(result, Some(GraphicsCapability::Kitty));
    }

    #[test]
    fn test_warning_message() {
        let warning = get_tui_fallback_warning();
        assert!(warning.contains("TUI fallback"));
        assert!(warning.contains("Kitty"));
    }

    #[test]
    fn test_detector_default() {
        let detector: CapabilityDetector = Default::default();
        let _capability = detector.detect();
        // Should not panic
    }
}
