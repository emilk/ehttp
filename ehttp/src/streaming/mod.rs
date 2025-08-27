//! Streaming HTTP client for both native and WASM.
//!
//! Requires the `streaming` feature to be enabled.
//!
//! Example:
//! ```
//! use std::time::Duration;
//!
//! let your_chunk_handler = std::sync::Arc::new(|chunk: Vec<u8>| {
//!     if chunk.is_empty() {
//!         return ehttp::streaming::Flow::Break;
//!     }
//!
//!     println!("received chunk: {} bytes", chunk.len());
//!     
//!     // Example of back-pressure: wait if chunk is large
//!     if chunk.len() > 1024 * 1024 {
//!         ehttp::streaming::Flow::Wait(Duration::from_millis(100))
//!     } else {
//!         ehttp::streaming::Flow::Continue
//!     }
//! });
//!
//! let url = "https://www.example.com";
//! let request = ehttp::Request::get(url);
//! ehttp::streaming::fetch(request, move |result: ehttp::Result<ehttp::streaming::Part>| {
//!     let part = match result {
//!         Ok(part) => part,
//!         Err(err) => {
//!             eprintln!("an error occurred while streaming `{url}`: {err}");
//!             return ehttp::streaming::Flow::Break;
//!         }
//!     };
//!
//!     match part {
//!         ehttp::streaming::Part::Response(response) => {
//!             println!("Status code: {:?}", response.status);
//!             if response.ok {
//!                 ehttp::streaming::Flow::Continue
//!             } else {
//!                 ehttp::streaming::Flow::Break
//!             }
//!         }
//!         ehttp::streaming::Part::Chunk(chunk) => {
//!             your_chunk_handler(chunk)
//!         }
//!     }
//! });
//! ```
//!
//! The streaming fetch works like the non-streaming fetch, but instead
//! of receiving the response in full, you receive parts of the response
//! as they are streamed in. The callback can return [`Flow::Wait`] to implement
//! back-pressure by pausing the stream for a specified duration.

use crate::Request;

/// Performs a HTTP requests and calls the given callback once for the initial response,
/// and then once for each chunk in the response body.
///
/// You can abort the fetch by returning [`types::Flow::Break`] from the callback,
/// or implement back-pressure by returning [`types::Flow::Wait`] with a duration.
pub fn fetch(
    request: Request,
    on_data: impl 'static + Send + Fn(crate::Result<types::Part>) -> types::Flow,
) {
    #[cfg(not(target_arch = "wasm32"))]
    native::fetch_streaming(request, Box::new(on_data));

    #[cfg(target_arch = "wasm32")]
    web::fetch_streaming(request, Box::new(on_data));
}

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::fetch_streaming_blocking;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::fetch_async_streaming;

mod types;

pub use self::types::{Flow, Part};
