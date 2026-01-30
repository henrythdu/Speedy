//! Terminal capability detection for graphics protocol support
//!
//! Detects Kitty Graphics Protocol support. Kitty-compatible terminals
//! are REQUIRED for Speedy to run.

use std::env;

/// Graphics protocol support levels - Kitty-only mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicsCapability {
    /// Kitty Graphics Protocol supported (REQUIRED for Speedy)
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
    /// EXITS with error message if Kitty Graphics Protocol not detected.
    /// Speedy REQUIRES Kitty-compatible terminal (Kitty or Konsole 22.04+).
    pub fn detect(&self) -> GraphicsCapability {
        // Check environment variables for Kitty support
        if let Some(capability) = self.detect_from_env() {
            return capability;
        }

        // Kitty Graphics Protocol not detected - exit with error
        panic!(
            "Speedy requires Kitty Graphics Protocol. \
            Please run Speedy in Kitty or Konsole 22.04+. \
            Set $TERM=xterm-kitty or $TERM_PROGRAM=kitty if needed."
        );
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

        // Check $KONSOLE_VERSION - Konsole sets this even when $TERM=xterm-256color
        if let Ok(_konsole_version) = env::var("KONSOLE_VERSION") {
            return Some(GraphicsCapability::Kitty);
        }

        None
    }

    /// Detect capability from explicit CLI override
    ///
    /// Only `force_kitty` parameter is supported (TUI fallback removed).
    pub fn detect_from_override(&self, force_kitty: bool) -> GraphicsCapability {
        if force_kitty {
            return GraphicsCapability::Kitty;
        }
        self.detect()
    }
}

impl Default for CapabilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_capability_kitty() {
        assert!(GraphicsCapability::Kitty.supports_graphics());
        assert!(GraphicsCapability::Kitty.supports_subpixel_ovp());
    }

    #[test]
    #[should_panic(expected = "Speedy requires Kitty Graphics Protocol")]
    fn test_detector_panics_on_no_kitty() {
        let detector = CapabilityDetector::new();
        // Clear all environment variables that would indicate Kitty support
        std::env::remove_var("TERM");
        std::env::remove_var("TERM_PROGRAM");
        std::env::remove_var("KONSOLE_VERSION");

        let _capability = detector.detect();
    }

    #[test]
    fn test_detect_from_override_force_kitty() {
        let detector = CapabilityDetector::new();
        let result = detector.detect_from_override(true);
        assert_eq!(result, GraphicsCapability::Kitty);
    }

    #[test]
    fn test_detect_konsole_via_konsole_version_env() {
        // Konsole sets $KONSOLE_VERSION even when $TERM=xterm-256color
        let detector = CapabilityDetector::new();
        std::env::set_var("TERM", "xterm-256color");
        std::env::set_var("KONSOLE_VERSION", "220400");
        let result = detector.detect_from_env();
        assert_eq!(result, Some(GraphicsCapability::Kitty));
        // Cleanup
        std::env::remove_var("KONSOLE_VERSION");
        std::env::remove_var("TERM");
    }

    #[test]
    fn test_detector_default() {
        let detector: CapabilityDetector = Default::default();
        // Set up Kitty environment to prevent panic
        std::env::set_var("TERM", "xterm-kitty");
        let _capability = detector.detect();
        // Cleanup
        std::env::remove_var("TERM");
    }
}
