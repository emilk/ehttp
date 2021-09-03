use std::collections::BTreeMap;

/// A simple http request.
pub struct Request {
    /// "GET", …
    pub method: String,
    /// https://…
    pub url: String,
    /// The raw bytes.
    pub body: Vec<u8>,
    /// ("Accept", "*/*"), …
    pub headers: BTreeMap<String, String>,
}

impl Request {
    pub fn create_headers_map(headers: &[(&str, &str)]) -> BTreeMap<String, String> {
        headers
            .iter()
            .map(|e| (e.0.to_owned(), e.1.to_owned()))
            .collect()
    }

    /// Create a `GET` request with the given url.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get(url: impl ToString) -> Self {
        Self {
            method: "GET".to_owned(),
            url: url.to_string(),
            body: vec![],
            headers: Request::create_headers_map(&[("Accept", "*/*")]),
        }
    }

    /// Create a `POST` request with the given url and body.
    #[allow(clippy::needless_pass_by_value)]
    pub fn post(url: impl ToString, body: impl ToString) -> Self {
        Self {
            method: "POST".to_owned(),
            url: url.to_string(),
            body: body.to_string().into_bytes(),
            headers: Request::create_headers_map(&[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ]),
        }
    }
}

/// Response from a completed HTTP request.
pub struct Response {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,
    /// Did we get a 2xx response code?
    pub ok: bool,
    /// Status code (e.g. `404` for "File not found").
    pub status: u16,
    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,
    /// The raw bytes.
    pub bytes: Vec<u8>,

    pub headers: BTreeMap<String, String>,
}

impl Response {
    pub fn text(&self) -> Option<String> {
        String::from_utf8(self.bytes.clone()).ok()
    }

    pub fn content_type(&self) -> Option<String> {
        self.headers.get("content-type").cloned()
    }
}

/// Possible errors does NOT include e.g. 404, which is NOT considered an error.
pub type Error = String;
