use std::ops::ControlFlow;

use crate::{Method, Request};

use super::Part;
use crate::types::PartialResponse;

pub fn fetch_streaming_blocking(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> ControlFlow<()> + Send>,
) {
    let resp = if request.method.contains_body() {
        let mut req = match request.method {
            Method::POST => ureq::post(&request.url),
            Method::PATCH => ureq::patch(&request.url),
            Method::PUT => ureq::put(&request.url),
            // These three are the only requests which contain a body, no other requests will be matched
            _ => unreachable!(), // because of the `.contains_body()` call
        };

        for (k, v) in &request.headers {
            req = req.header(k, v);
        }

        req = req.config().http_status_as_error(false).build();

        if request.body.is_empty() {
            req.send_empty()
        } else {
            req.send(&request.body)
        }
    } else {
        let mut req = match request.method {
            Method::GET => ureq::get(&request.url),
            Method::DELETE => ureq::delete(&request.url),
            Method::CONNECT => ureq::connect(&request.url),
            Method::HEAD => ureq::head(&request.url),
            Method::OPTIONS => ureq::options(&request.url),
            Method::TRACE => ureq::trace(&request.url),
            // Include all other variants rather than a catch all here to prevent confusion if another variant were to be added
            Method::PATCH | Method::POST | Method::PUT => unreachable!(), // because of the `.contains_body()` call
        };

        req = req.config().http_status_as_error(false).build();

        for (k, v) in &request.headers {
            req = req.header(k, v);
        }

        if request.body.is_empty() {
            req.call()
        } else {
            req.force_send_body().send(&request.body)
        }
    };

    let mut resp = match resp {
        Ok(t) => t,
        Err(e) => {
            on_data(Err(e.to_string()));
            return;
        }
    };

    let ok = resp.status().is_success();
    use ureq::ResponseExt as _;
    let url = resp.get_uri().to_string();
    let status = resp.status().as_u16();
    let status_text = resp
        .status()
        .canonical_reason()
        .unwrap_or("Error")
        .to_string();
    let mut headers = crate::Headers::default();
    for (k, v) in resp.headers().iter() {
        headers.insert(
            k,
            match v.to_str() {
                Ok(t) => t,
                Err(e) => {
                    on_data(Err(e.to_string()));
                    break;
                }
            },
        );
    }
    headers.sort(); // It reads nicer, and matches web backend.

    let response = PartialResponse {
        url,
        ok,
        status,
        status_text,
        headers,
    };
    if on_data(Ok(Part::Response(response))).is_break() {
        return;
    };

    let mut reader = resp.body_mut().as_reader();
    loop {
        let mut buf = vec![0; 2048];
        use std::io::Read;
        match reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                // clone data from buffer and clear it
                let chunk = buf[..n].to_vec();
                if on_data(Ok(Part::Chunk(chunk))).is_break() {
                    return;
                };
            }
            Ok(_) => {
                on_data(Ok(Part::Chunk(vec![])));
                break;
            }
            Err(err) => {
                if request.method == Method::HEAD && err.kind() == std::io::ErrorKind::UnexpectedEof
                {
                    // We don't really expect a body for HEAD requests, so this is fine.
                    on_data(Ok(Part::Chunk(vec![])));
                    break;
                } else {
                    on_data(Err(format!("Failed to read response body: {err}")));
                    return;
                }
            }
        };
    }
}

pub(crate) fn fetch_streaming(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> ControlFlow<()> + Send>,
) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || fetch_streaming_blocking(request, on_data))
        .expect("Failed to spawn ehttp thread");
}
