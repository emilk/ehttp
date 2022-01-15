//! Minimal HTTP client for both native and WASM.

/// Performs a HTTP requests and calls the given callback when done.
pub fn fetch(request: Request, on_done: impl 'static + Send + FnOnce(Result<Response>)) {
    #[cfg(not(target_arch = "wasm32"))]
    native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    web::fetch(request, Box::new(on_done));
}

mod types;
pub use types::{Error, Request, Response, Result};

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::fetch_blocking;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::{fetch_async, spawn_future};

/// Helper for constructing [`Request::headers`].
/// ```
/// use ehttp::Request;
/// let request = Request {
///     headers: ehttp::headers(&[
///         ("Accept", "*/*"),
///         ("Content-Type", "text/plain; charset=utf-8"),
///     ]),
///     ..Request::get("https://www.example.com")
/// };
/// ```
pub fn headers(headers: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
    headers
        .iter()
        .map(|e| (e.0.to_owned(), e.1.to_owned()))
        .collect()
}
