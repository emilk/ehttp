use std::ops::ControlFlow;
use std::sync::Arc;

use crate::Request;

use super::Part;
use crate::types::PartialResponse;

pub fn fetch_streaming_blocking(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> ControlFlow<()> + Send>,
) {
    let tls_connector = Arc::new(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_hostnames(request.danger_accept_invalid_hostnames)
            .danger_accept_invalid_certs(request.danger_accept_invalid_certs)
            .build()
            .unwrap(),
    );

    let mut req = ureq::builder()
        .tls_connector(tls_connector)
        .build()
        .request(&request.method, &request.url);

    for (k, v) in &request.headers {
        req = req.set(k, v);
    }

    let resp = if request.body.is_empty() {
        req.call()
    } else {
        req.send_bytes(&request.body)
    };

    let (ok, resp) = match resp {
        Ok(resp) => (true, resp),
        Err(ureq::Error::Status(_, resp)) => (false, resp), // Still read the body on e.g. 404
        Err(ureq::Error::Transport(err)) => {
            on_data(Err(err.to_string()));
            return;
        }
    };

    let url = resp.get_url().to_owned();
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let mut headers = crate::Headers::default();
    for key in &resp.headers_names() {
        if let Some(value) = resp.header(key) {
            headers.insert(key.to_ascii_lowercase(), value.to_owned());
        }
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

    let mut reader = resp.into_reader();
    loop {
        let mut buf = vec![0; 5120];
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
                if request.method == "HEAD" && err.kind() == std::io::ErrorKind::UnexpectedEof {
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
