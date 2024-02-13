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
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
//!

/// Performs an HTTP request and calls the given callback when done.
///
/// `Ok` is returned if we get a response, even if it's a 404.
///
/// `Err` can happen for a number of reasons:
/// * No internet connection
/// * DNS resolution failed
/// * Firewall or proxy blocked the request
/// * Server is not reachable
/// * The URL is invalid
/// * Server's SSL cert is invalid
/// * CORS errors
/// * The initial GET which returned HTML contained CSP headers to block access to the resource
/// * A browser extension blocked the request (e.g. ad blocker)
/// * …
pub fn fetch(request: Request, on_done: impl 'static + Send + FnOnce(Result<Response>)) {
    #[cfg(not(target_arch = "wasm32"))]
    native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    web::fetch(request, Box::new(on_done));
}

/// Performs an `async` HTTP request.
///
/// Available on following platforms:
/// - web
/// - native behind the `native-async` feature.
///
/// `Ok` is returned if we get a response, even if it's a 404.
///
/// `Err` can happen for a number of reasons:
/// * No internet connection
/// * DNS resolution failed
/// * Firewall or proxy blocked the request
/// * Server is not reachable
/// * The URL is invalid
/// * Server's SSL cert is invalid
/// * CORS errors
/// * The initial GET which returned HTML contained CSP headers to block access to the resource
/// * A browser extension blocked the request (e.g. ad blocker)
/// * …
#[cfg(any(target_arch = "wasm32", feature = "native-async"))]
pub async fn fetch_async(request: Request) -> Result<Response> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(request).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request).await;
}

mod types;
pub use types::{Error, Headers, PartialResponse, Request, Response, Result};

#[cfg(target_arch = "wasm32")]
pub use types::Mode;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::fetch_blocking;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::spawn_future;

#[cfg(feature = "streaming")]
pub mod streaming;

#[cfg(feature = "multipart")]
pub mod multipart;

#[deprecated = "Use ehttp::Headers::new"]
pub fn headers(headers: &[(&str, &str)]) -> Headers {
    Headers::new(headers)
}
