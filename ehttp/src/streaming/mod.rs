use std::ops::ControlFlow;

use crate::Request;

/// Performs a HTTP requests and calls the given callback once for the initial response,
/// and then once for each chunk in the response body.
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
