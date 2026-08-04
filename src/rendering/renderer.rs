//! RsvpRenderer trait definition for Kitty Graphics Protocol rendering
//!
//! This trait provides pixel-level word rendering with OVP anchoring
//! and optional vertical ghost words for eye tracking continuity.

use std::error::Error;
use std::fmt;

/// A single frame to render with optional ghost context
///
/// Ghost words (previous and next) provide eye tracking continuity and
/// comprehension preview during speed reading. All three words' anchor
/// letters share the same X coordinate (ORP center).
///
/// # Example
/// ```
/// use speedy::rendering::renderer::RenderFrame;
///
/// let frame = RenderFrame {
///     word: "hello",
///     anchor: 2,
///     ghost_prev: Some(("world", 2)),
///     ghost_next: Some(("rust", 1)),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RenderFrame<'a> {
    /// Current word being displayed
    pub word: &'a str,
    /// Anchor index in current word (character position at ORP center)
    pub anchor: usize,
    /// Previous word ghost (word text, anchor position)
    pub ghost_prev: Option<(&'a str, usize)>,
    /// Next word ghost (word text, anchor position)
    pub ghost_next: Option<(&'a str, usize)>,
}

impl<'a> RenderFrame<'a> {
    /// Create a frame with ghost context
    pub fn with_ghosts(
        word: &'a str,
        anchor: usize,
        ghost_prev: Option<(&'a str, usize)>,
        ghost_next: Option<(&'a str, usize)>,
    ) -> Self {
        Self {
            word,
            anchor,
            ghost_prev,
            ghost_next,
        }
    }
}

/// Errors that can occur during renderer operations
#[derive(Debug, Clone, PartialEq)]
pub enum RendererError {
    /// Failed to initialize renderer resources
    InitializationFailed(String),
    /// Failed to render word
    RenderFailed(String),
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
            Self::CleanupFailed(msg) => write!(f, "Renderer cleanup failed: {}", msg),
            Self::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl Error for RendererError {}

/// Core trait for RSVP word rendering backends
///
/// Implementations handle the actual display of words, abstracting away
/// whether we're using TUI cell-based rendering or kitty pixel graphics.
pub trait RsvpRenderer {
    /// Initialize the renderer (allocate resources, setup state)
    ///
    /// Called once at app startup. May fail if resources unavailable.
    fn initialize(&mut self) -> Result<(), RendererError>;

    /// Render a complete frame with optional ghost words
    ///
    /// Ghost words provide eye tracking continuity:
    /// - `ghost_prev`: Previous word displayed above current (faded)
    /// - `ghost_next`: Next word displayed below current (faded)
    ///
    /// All three words' anchor letters share the same X coordinate (ORP center),
    /// keeping the eye fixed horizontally.
    ///
    /// # Render Order (Critical for Partial-Render UX)
    /// 1. Previous ghost (above) - transmitted first, non-fatal if fails
    /// 2. Next ghost (below) - transmitted second, non-fatal if fails
    /// 3. Current word (center) - transmitted last, MUST succeed
    ///
    /// This order prioritizes the current word if transmission is incomplete.
    ///
    /// # Arguments
    /// * `frame` - The frame to render containing current word and optional ghosts
    ///
    /// # Errors
    /// Returns `RendererError::InvalidArguments` if anchor positions are out of bounds.
    /// Ghost render failures should be logged but NOT propagated (non-fatal).
    fn render_frame(&mut self, frame: &RenderFrame) -> Result<(), RendererError>;

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

        fn render_frame(&mut self, frame: &RenderFrame) -> Result<(), RendererError> {
            // Validate current word anchor
            if frame.anchor >= frame.word.chars().count() {
                return Err(RendererError::InvalidArguments(format!(
                    "anchor {} out of bounds for word '{}'",
                    frame.anchor, frame.word
                )));
            }
            // Validate ghost anchors (non-fatal in real impl, but validate for tests)
            if let Some((word, anchor)) = frame.ghost_prev {
                if anchor >= word.chars().count() {
                    return Err(RendererError::InvalidArguments(format!(
                        "ghost_prev anchor {} out of bounds for word '{}'",
                        anchor, word
                    )));
                }
            }
            if let Some((word, anchor)) = frame.ghost_next {
                if anchor >= word.chars().count() {
                    return Err(RendererError::InvalidArguments(format!(
                        "ghost_next anchor {} out of bounds for word '{}'",
                        anchor, word
                    )));
                }
            }
            Ok(())
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
        let mut renderer = TestRenderer;

        assert!(renderer.initialize().is_ok());
        let frame = RenderFrame::with_ghosts("hello", 1, None, None);
        assert!(renderer.render_frame(&frame).is_ok());
        assert!(renderer.cleanup().is_ok());
    }

    #[test]
    fn test_render_frame_validates_anchor_position() {
        let mut renderer = TestRenderer;

        let frame = RenderFrame::with_ghosts("hello", 0, None, None);
        assert!(renderer.render_frame(&frame).is_ok());
        let frame = RenderFrame::with_ghosts("hello", 4, None, None);
        assert!(renderer.render_frame(&frame).is_ok());

        let frame = RenderFrame::with_ghosts("hi", 5, None, None);
        assert!(renderer.render_frame(&frame).is_err());
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

    #[test]
    fn test_render_frame_with_ghosts() {
        let frame =
            RenderFrame::with_ghosts("current", 3, Some(("previous", 4)), Some(("next", 1)));
        assert_eq!(frame.word, "current");
        assert_eq!(frame.anchor, 3);
        assert_eq!(frame.ghost_prev, Some(("previous", 4)));
        assert_eq!(frame.ghost_next, Some(("next", 1)));
    }

    #[test]
    fn test_render_frame_validates_anchors() {
        let mut renderer = TestRenderer;

        // Valid frame with ghosts
        let frame = RenderFrame::with_ghosts("hello", 2, Some(("prev", 1)), Some(("next", 2)));
        assert!(renderer.render_frame(&frame).is_ok());

        // Invalid current word anchor
        let frame = RenderFrame::with_ghosts("hi", 5, None, None);
        assert!(renderer.render_frame(&frame).is_err());

        // Invalid ghost_prev anchor
        let frame = RenderFrame::with_ghosts("word", 1, Some(("a", 5)), None);
        assert!(renderer.render_frame(&frame).is_err());

        // Invalid ghost_next anchor
        let frame = RenderFrame::with_ghosts("word", 1, None, Some(("abc", 10)));
        assert!(renderer.render_frame(&frame).is_err());
    }

    #[test]
    fn test_render_frame_trait_object() {
        // Verify render_frame works through trait object
        let mut renderer: Box<dyn RsvpRenderer> = Box::new(TestRenderer);
        let frame = RenderFrame::with_ghosts("test", 1, None, None);
        assert!(renderer.render_frame(&frame).is_ok());
    }
}
