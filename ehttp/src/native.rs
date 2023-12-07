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
    let mut req = ureq::request(&request.method, &request.url);

    for header in &request.headers {
        req = req.set(header.0.as_str(), header.1.as_str());
    }

    let resp = if request.body.is_empty() {
        req.call()
    } else {
        req.send_bytes(&request.body)
    };

    let (ok, resp) = match resp {
        Ok(resp) => (true, resp),
        Err(ureq::Error::Status(_, resp)) => (false, resp), // Still read the body on e.g. 404
        Err(ureq::Error::Transport(error)) => return Err(error.to_string()),
    };

    let url = resp.get_url().to_owned();
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let mut headers = vec![];
    for key in &resp.headers_names() {
        if let Some(value) = resp.header(key) {
            // lowercase for easy lookup
            headers.push((key.to_ascii_lowercase(), value.to_owned()));
        }
    }

    let mut reader = resp.into_reader();
    let mut bytes = vec![];
    use std::io::Read;
    reader
        .read_to_end(&mut bytes)
        .map_err(|err| err.to_string())?;

    let response = Response {
        url,
        ok,
        status,
        status_text,
        bytes,
        headers,
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
