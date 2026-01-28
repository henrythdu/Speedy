//! RsvpRenderer trait definition for pluggable rendering backends
//!
//! This trait abstracts both TUI (CellRenderer) and graphics (Kitty) backends,
//! enabling future support for Sixel, iTerm2, and other protocols.

use std::error::Error;
use std::fmt;

/// Errors that can occur during renderer operations
#[derive(Debug, Clone, PartialEq)]
pub enum RendererError {
    /// Failed to initialize renderer resources
    InitializationFailed(String),
    /// Failed to render word
    RenderFailed(String),
    /// Failed to clear display
    ClearFailed(String),
    /// Failed to cleanup resources
    CleanupFailed(String),
    /// Invalid arguments provided
    InvalidArguments(String),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "Renderer initialization failed: {}", msg),
            Self::RenderFailed(msg) => write!(f, "Word rendering failed: {}", msg),
            Self::ClearFailed(msg) => write!(f, "Clear operation failed: {}", msg),
            Self::CleanupFailed(msg) => write!(f, "Renderer cleanup failed: {}", msg),
            Self::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl Error for RendererError {}

/// Core trait for RSVP word rendering backends
///
/// Implementations handle the actual display of words, abstracting away
/// whether we're using TUI cell-based rendering or pixel-perfect graphics.
pub trait RsvpRenderer {
    /// Initialize the renderer (allocate resources, setup state)
    ///
    /// Called once at app startup. May fail if resources unavailable.
    fn initialize(&mut self) -> Result<(), RendererError>;

    /// Render a single word at the current position
    ///
    /// The word is positioned according to OVP (Optimal Viewing Position) anchoring.
    /// In graphics mode: sub-pixel accurate positioning
    /// In TUI mode: snaps to nearest character cell
    ///
    /// # Arguments
    /// * `word` - The word to render
    /// * `anchor_position` - Character index within word that should be at OVP (0-based)
    ///
    /// # Errors
    /// Returns `RendererError::InvalidArguments` if `anchor_position` is out of bounds
    /// for the given word.
    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError>;

    /// Clear the current word from the display
    ///
    /// Removes any previously rendered content in the reading zone.
    fn clear(&mut self) -> Result<(), RendererError>;

    /// Check if this renderer supports sub-pixel OVP positioning
    ///
    /// Returns true for graphics backends (Kitty), false for TUI backends.
    fn supports_subpixel_ovp(&self) -> bool;

    /// Cleanup resources before app exit
    ///
    /// Ensures no lingering graphics or state remains.
    fn cleanup(&mut self) -> Result<(), RendererError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stub implementation for testing trait object safety
    struct TestRenderer;

    impl RsvpRenderer for TestRenderer {
        fn initialize(&mut self) -> Result<(), RendererError> {
            Ok(())
        }

        fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError> {
            if anchor_position >= word.chars().count() {
                return Err(RendererError::InvalidArguments(format!(
                    "anchor_position {} out of bounds for word '{}'",
                    anchor_position, word
                )));
            }
            Ok(())
        }

        fn clear(&mut self) -> Result<(), RendererError> {
            Ok(())
        }

        fn supports_subpixel_ovp(&self) -> bool {
            false
        }

        fn cleanup(&mut self) -> Result<(), RendererError> {
            Ok(())
        }
    }

    #[test]
    fn test_trait_object_safety() {
        // This test verifies the trait is object-safe (can use Box<dyn RsvpRenderer>)
        let _renderer: Box<dyn RsvpRenderer> = Box::new(TestRenderer);
    }

    #[test]
    fn test_stub_implementation_compiles() {
        // Verify stub implementation exists and methods are callable
        let mut renderer = TestRenderer;

        assert!(renderer.initialize().is_ok());
        assert!(renderer.render_word("hello", 1).is_ok());
        assert!(renderer.clear().is_ok());
        assert!(!renderer.supports_subpixel_ovp());
        assert!(renderer.cleanup().is_ok());
    }

    #[test]
    fn test_render_word_validates_anchor_position() {
        let mut renderer = TestRenderer;

        // Valid anchor positions should work
        assert!(renderer.render_word("hello", 0).is_ok());
        assert!(renderer.render_word("hello", 4).is_ok());

        // Out of bounds should return error
        let result = renderer.render_word("hi", 5);
        assert!(result.is_err());
        match result {
            Err(RendererError::InvalidArguments(_)) => (), // Expected
            _ => panic!("Expected InvalidArguments error"),
        }
    }

    #[test]
    fn test_error_display_messages() {
        let err = RendererError::InitializationFailed("test".to_string());
        assert!(err.to_string().contains("initialization failed"));

        let err = RendererError::RenderFailed("test".to_string());
        assert!(err.to_string().contains("rendering failed"));

        let err = RendererError::InvalidArguments("test".to_string());
        assert!(err.to_string().contains("Invalid arguments"));
    }
}
