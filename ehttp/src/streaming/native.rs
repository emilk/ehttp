use std::ops::ControlFlow;

use crate::{Method, Request};

use super::Part;
use crate::types::PartialResponse;

pub fn fetch_streaming_blocking(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> ControlFlow<()> + Send>,
) {
    let resp = request.fetch_raw_native(false);

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
        .unwrap_or("ERROR")
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
                if err.kind() == std::io::ErrorKind::Other && request.method == Method::HEAD {
                    match err.downcast::<ureq::Error>() {
                        Ok(ureq::Error::Decompress(_, io_err))
                            if io_err.kind() == std::io::ErrorKind::UnexpectedEof =>
                        {
                            // We don't really expect a body for HEAD requests, so this is fine.
                            on_data(Ok(Part::Chunk(vec![])));
                            break;
                        }
                        Ok(err_inner) => {
                            on_data(Err(format!("Failed to read response body: {err_inner}")));
                            return;
                        }
                        Err(err) => {
                            on_data(Err(format!("Failed to read response body: {err}")));
                            return;
                        }
                    }
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
