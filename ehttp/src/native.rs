use crate::{Request, Response};

#[cfg(feature = "native-async")]
use async_channel::{Receiver, Sender};

/// Performs a  HTTP request and blocks the thread until it is done.
///
/// Only available when compiling for native.
///
/// NOTE: `Ok(…)` is returned on network error.
///
/// `Ok` is returned if we get a response, even if it's a 404.
///
/// `Err` can happen for a number of reasons:
/// * No internet connection
/// * Connection timed out
/// * DNS resolution failed
/// * Firewall or proxy blocked the request
/// * Server is not reachable
/// * The URL is invalid
/// * Server's SSL cert is invalid
/// * CORS errors
/// * The initial GET which returned HTML contained CSP headers to block access to the resource
/// * A browser extension blocked the request (e.g. ad blocker)
/// * …
pub fn fetch_blocking(request: &Request) -> crate::Result<Response> {
    let resp = match request.method.as_str() {
        "POST" | "PATCH" | "PUT" => {
            let mut req = match request.method.as_str() {
                "POST" => ureq::post(&request.url),
                "PATCH" => ureq::patch(&request.url),
                "PUT" => ureq::put(&request.url),
                _ => unreachable!(),
            };

            for (k, v) in &request.headers {
                req = req.header(k, v);
            }

            req = req
                .config()
                .timeout_recv_body(request.timeout)
                .http_status_as_error(false)
                .build();

            if request.body.is_empty() {
                req.send_empty()
            } else {
                req.send(&request.body)
            }
        }
        "GET" | "DELETE" | "CONNECT" | "HEAD" | "OPTIONS" | "TRACE" => {
            let mut req = match request.method.as_str() {
                "GET" => ureq::get(&request.url),
                "DELETE" => ureq::delete(&request.url),
                "CONNECT" => ureq::connect(&request.url),
                "HEAD" => ureq::head(&request.url),
                "OPTIONS" => ureq::options(&request.url),
                "TRACE" => ureq::trace(&request.url),
                _ => unreachable!(),
            };

            req = req
                .config()
                .timeout_recv_body(request.timeout)
                .http_status_as_error(false)
                .build();

            for (k, v) in &request.headers {
                req = req.header(k, v);
            }

            if request.body.is_empty() {
                req.call()
            } else {
                req.force_send_body().send(&request.body)
            }
        }
        _ => return Err(String::from("Failed to parse request method")),
    };

    let mut resp = resp.map_err(|err| err.to_string())?;

    let ok = resp.status().is_success();
    use ureq::ResponseExt as _;
    let url = resp.get_uri().to_string();
    let status = resp.status().as_u16();
    let status_text = resp
        .status()
        .canonical_reason()
        .unwrap_or("ERROR")
        .to_string();
    let mut headers = crate::Headers::default();
    for (k, v) in resp.headers().iter() {
        headers.insert(
            k,
            v.to_str()
                .map_err(|e| format!("Failed to convert header value to string: {e}"))?,
        );
    }
    headers.sort(); // It reads nicer, and matches web backend.

    let mut reader = resp.body_mut().as_reader();
    let mut bytes = vec![];
    use std::io::Read as _;
    if let Err(err) = reader.read_to_end(&mut bytes) {
        if request.method == "HEAD" && err.kind() == std::io::ErrorKind::UnexpectedEof {
            // We don't really expect a body for HEAD requests, so this is fine.
        } else {
            return Err(format!("Failed to read response body: {err}"));
        }
    }

    let response = Response {
        url,
        ok,
        status,
        status_text,
        headers,
        bytes,
    };
    Ok(response)
}

// ----------------------------------------------------------------------------

pub(crate) fn fetch(request: Request, on_done: Box<dyn FnOnce(crate::Result<Response>) + Send>) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || on_done(fetch_blocking(&request)))
        .expect("Failed to spawn ehttp thread");
}

#[cfg(feature = "native-async")]
pub(crate) async fn fetch_async(request: Request) -> crate::Result<Response> {
    let (tx, rx): (
        Sender<crate::Result<Response>>,
        Receiver<crate::Result<Response>>,
    ) = async_channel::bounded(1);

    fetch(
        request,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv().await.map_err(|err| err.to_string())?
}
