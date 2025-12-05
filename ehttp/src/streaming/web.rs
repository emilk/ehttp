use futures_util::Stream;
use futures_util::StreamExt;
use wasm_bindgen::prelude::*;

use crate::web::{fetch_base, get_response_base, spawn_future, string_from_fetch_error};
use crate::Request;

use super::types::{Flow, Part};

/// Only available when compiling for web.
///
/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
#[cfg(feature = "streaming")]
pub async fn fetch_async_streaming(
    request: &Request,
) -> crate::Result<impl Stream<Item = crate::Result<Part>>> {
    let stream = fetch_jsvalue_stream(request)
        .await
        .map_err(string_from_fetch_error)?;
    Ok(stream.map(|result| result.map_err(string_from_fetch_error)))
}

#[cfg(feature = "streaming")]
async fn fetch_jsvalue_stream(
    request: &Request,
) -> Result<impl Stream<Item = Result<Part, JsValue>>, JsValue> {
    use js_sys::Uint8Array;

    let response = fetch_base(request).await?;
    let body = wasm_streams::ReadableStream::from_raw(
        response.body().ok_or("response has no body")?.dyn_into()?,
    );

    // returns a `Part::Response` followed by all the chunks in `body` as `Part::Chunk`
    Ok(
        futures_util::stream::once(futures_util::future::ready(Ok(Part::Response(
            get_response_base(&response)?,
        ))))
        .chain(
            body.into_stream()
                .map(|value| value.map(|value| Part::Chunk(Uint8Array::new(&value).to_vec()))),
        ),
    )
}

async fn sleep(duration: std::time::Duration) {
    // Ignore all errors

    let millis = duration.as_millis() as _;
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        if let Some(window) = web_sys::window() {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis)
                .ok();
        }
    };
    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.ok();
}

pub(crate) fn fetch_streaming(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> Flow + Send>,
) {
    spawn_future(async move {
        let mut stream = match fetch_jsvalue_stream(&request).await {
            Ok(stream) => stream,
            Err(e) => {
                on_data(Err(string_from_fetch_error(e)));
                return;
            }
        };

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => match on_data(Ok(chunk)) {
                    Flow::Break => return,
                    Flow::Wait(duration) => sleep(duration).await,
                    Flow::Continue => {}
                },
                Err(e) => {
                    on_data(Err(string_from_fetch_error(e)));
                    return;
                }
            }
        }

        on_data(Ok(Part::Chunk(vec![])));
    })
}
