//! Minimal HTTP client for both native and WASM.
//!
//! Example:
//! ```
//! let request = ehttp::Request::get("https://www.example.com");
//! ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
//!     println!("Status code: {:?}", result.unwrap().status);
//! });
//! ```
//!
//! The given callback is called when the request is completed.
//! You can communicate the results back to the main thread using something like:
//!
//! * Channels (e.g. [`std::sync::mpsc::channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html)).
//! * `Arc<Mutex<_>>`
//! * [`poll_promise::Promise`](https://docs.rs/poll-promise)
//! * [`eventuals::Eventual`](https://docs.rs/eventuals/latest/eventuals/struct.Eventual.html)
//! * [`tokio::sync::watch::channel`](https://docs.rs/tokio/latest/tokio/sync/watch/fn.channel.html)

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
