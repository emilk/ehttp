use futures_util::Stream;
use futures_util::StreamExt;
use wasm_bindgen::prelude::*;

use super::types::Response;
use crate::web::{fetch_base, get_response_base, spawn_future, string_from_js_value};
use crate::Request;

use super::types::Part;

/// Only available when compiling for web.
///
/// NOTE: Ok(…) is returned on network error.
/// Err is only for failure to use the fetch api.
#[cfg(feature = "streaming")]
pub async fn fetch_async_streaming(
    request: &Request,
) -> crate::Result<impl Stream<Item = crate::Result<Part>>> {
    let stream = fetch_jsvalue_stream(request)
        .await
        .map_err(string_from_js_value)?;
    Ok(stream.map(|result| result.map_err(string_from_js_value)))
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
            Response::from(get_response_base(&response)?),
        ))))
        .chain(
            body.into_stream()
                .map(|value| value.map(|value| Part::Chunk(Uint8Array::new(&value).to_vec()))),
        ),
    )
}

pub(crate) fn fetch_streaming(request: Request, on_data: Box<dyn Fn(crate::Result<Part>) + Send>) {
    spawn_future(async move {
        let mut stream = match fetch_jsvalue_stream(&request).await {
            Ok(stream) => stream,
            Err(e) => {
                return on_data(Err(string_from_js_value(e)));
            }
        };

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => on_data(Ok(chunk)),
                Err(e) => {
                    return on_data(Err(string_from_js_value(e)));
                }
            }
        }

        on_data(Ok(Part::Chunk(vec![])));
    })
}
