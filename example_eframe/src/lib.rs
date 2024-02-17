//! Example application using [`eframe`].

mod app;

pub use app::DemoApp;

// ----------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;
