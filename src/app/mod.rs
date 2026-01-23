#[allow(clippy::module_inception)]
pub mod app;

#[cfg(test)]
mod app_tests;
pub mod mode;

pub use app::{App, AppEvent, RenderState};
