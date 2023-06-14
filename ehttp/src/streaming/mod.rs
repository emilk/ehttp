//! Streaming HTTP client for both native and WASM.
//!
//! Example:
//! ```
//! let url = "https://www.example.com";
//! let request = ehttp::Request::get(url);
//! ehttp::streaming::fetch(request, move |result: ehttp::Result<ehttp::streaming::Part>| {
//!     let part = match result {
//!         Ok(part) => part,
//!         Err(err) => {
//!             eprintln!("an error occurred while streaming `{url}`: {err}");
//!             return std::ops::ControlFlow::Break(());
//!         }
//!     }
//!
//!     match part {
//!         ehttp::streaming::Part::Response(response) => {
//!             println!("Status code: {:?}", response.status);
//!             if !response.ok {
//!                 std::ops::ControlFlow::Break(())
//!             } else {
//!                 std::ops::ControlFlow::Continue(())
//!             }
//!         }
//!         ehttp::streaming::Part::Chunk(chunk) => {
//!             match your_chunk_handler(chunk) {
//!                 Ok(()) => std::ops::ControlFlow::Continue(()),
//!                 Err(err) => {
//!                     eprintln!("an error occurred while streaming `{url}`: {err}");
//!                     std::ops::ControlFlow::Break(())
//!                 }
//!             }
//!         }
//!     }
//! });
//! ```
//!
//! The streaming fetch works like the non-streaming fetch, but instead
//! of receiving the response in full, you receive parts of the response
//! as they are streamed in.

use std::ops::ControlFlow;

use crate::Request;

/// Performs a HTTP requests and calls the given callback once for the initial response,
/// and then once for each chunk in the response body.
///
/// You can abort the fetch by returning [`ControlFlow::Break`] from the callback.
pub fn fetch(
    request: Request,
    on_data: impl 'static + Send + Fn(crate::Result<types::Part>) -> ControlFlow<()>,
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

pub use self::types::Part;
