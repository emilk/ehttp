use std::ops::ControlFlow;
use std::vec;

use crate::Request;

use super::Part;
use crate::types::PartialResponse;

pub fn fetch_streaming_blocking(
    request: Request,
    on_data: Box<dyn Fn(crate::Result<Part>) -> ControlFlow<()> + Send>,
) {
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
        Err(ureq::Error::Transport(error)) => {
            on_data(Err(error.to_string()));
            return;
        }
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
        let mut buf = vec![0; 2048];
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
            Err(error) => {
                on_data(Err(error.to_string()));
                return;
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
