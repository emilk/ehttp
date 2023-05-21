use std::collections::BTreeMap;

/// A simple http request.
#[derive(Clone, Debug)]
pub struct Request {
    /// "GET", "POST", …
    pub method: String,

    /// https://…
    pub url: String,

    /// The data you send with e.g. "POST".
    pub body: Vec<u8>,

    /// ("Accept", "*/*"), …
    pub headers: BTreeMap<String, String>,
}

impl Request {
    /// Create a `GET` request with the given url.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get(url: impl ToString) -> Self {
        Self {
            method: "GET".to_owned(),
            url: url.to_string(),
            body: vec![],
            headers: crate::headers(&[("Accept", "*/*")]),
        }
    }

    /// Create a `POST` request with the given url and body.
    #[allow(clippy::needless_pass_by_value)]
    pub fn post(url: impl ToString, body: Vec<u8>) -> Self {
        Self {
            method: "POST".to_owned(),
            url: url.to_string(),
            body,
            headers: crate::headers(&[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ]),
        }
    }
}

/// Response from a completed HTTP request.
#[derive(Clone, Eq, PartialEq)]
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

    /// The returned headers. All header names are lower-case.
    pub headers: BTreeMap<String, String>,
}

impl Response {
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Response")
            .field("url", &self.url)
            .field("ok", &self.ok)
            .field("status", &self.status)
            .field("status_text", &self.status_text)
            //    .field("bytes", &self.bytes)
            .field("headers", &self.headers)
            .finish_non_exhaustive()
    }
}

/// A description of an error.
///
/// HTTP errors like 404 (file not found) are NOT considered errors.
pub type Error = String;

/// A type-alias for `Result<T, ehttp::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
