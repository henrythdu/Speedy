#[allow(clippy::module_inception)]
pub mod app;

#[cfg(test)]
mod app_tests;
pub mod mode;
pub mod state;

pub use app::{App, AppEvent};
