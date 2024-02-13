use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::types::PartialResponse;
use crate::{Request, Response};

/// Only available when compiling for web.
///
/// NOTE: `Ok(…)` is returned on network error.
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
pub async fn fetch_async(request: &Request) -> crate::Result<Response> {
    fetch_jsvalue(request)
        .await
        .map_err(string_from_fetch_error)
}

/// This should only be used to handle opaque exceptions thrown by the `fetch` call.
pub(crate) fn string_from_fetch_error(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| {
        // TypeError means that this is an opaque `network error`, as defined by the spec:
        // https://fetch.spec.whatwg.org/
        if value.has_type::<js_sys::TypeError>() {
            web_sys::console::error_1(&value);
            "Failed to fetch, check the developer console for details".to_owned()
        } else {
            format!("{:#?}", value)
        }
    })
}

pub(crate) async fn fetch_base(request: &Request) -> Result<web_sys::Response, JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(&request.method);
    opts.mode(request.mode.into());

    if !request.body.is_empty() {
        let body_bytes: &[u8] = &request.body;
        let body_array: js_sys::Uint8Array = body_bytes.into();
        let js_value: &JsValue = body_array.as_ref();
        opts.body(Some(js_value));
    }

    let js_request = web_sys::Request::new_with_str_and_init(&request.url, &opts)?;

    for (k, v) in &request.headers {
        js_request.headers().set(k, v)?;
    }

    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_request(&js_request)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    Ok(response)
}

pub(crate) fn get_response_base(response: &web_sys::Response) -> Result<PartialResponse, JsValue> {
    // https://developer.mozilla.org/en-US/docs/Web/API/Headers
    // "Note: When Header values are iterated over, […] values from duplicate header names are combined."
    // TODO: support duplicate header names
    let js_headers: web_sys::Headers = response.headers();
    let js_iter = js_sys::try_iter(&js_headers)
        .expect("headers try_iter")
        .expect("headers have an iterator");

    let mut headers = crate::Headers::default();
    for item in js_iter {
        let item = item.expect("headers iterator");
        let array: js_sys::Array = item.into();
        let v: Vec<JsValue> = array.to_vec();

        let key = v[0]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers name"))?;
        let value = v[1]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers value"))?;

        headers.insert(key, value);
    }

    Ok(PartialResponse {
        url: response.url(),
        ok: response.ok(),
        status: response.status(),
        status_text: response.status_text(),
        headers,
    })
}

/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
async fn fetch_jsvalue(request: &Request) -> Result<Response, JsValue> {
    let response = fetch_base(request).await?;

    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    let base = get_response_base(&response)?;

    Ok(Response {
        url: base.url,
        ok: base.ok,
        status: base.status,
        status_text: base.status_text,
        bytes,
        headers: base.headers,
    })
}

/// Spawn an async task.
///
/// A wrapper around `wasm_bindgen_futures::spawn_local`.
/// Only available with the web backend.
pub fn spawn_future<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

// ----------------------------------------------------------------------------

pub(crate) fn fetch(request: Request, on_done: Box<dyn FnOnce(crate::Result<Response>) + Send>) {
    spawn_future(async move {
        let result = fetch_async(&request).await;
        on_done(result)
    });
}
