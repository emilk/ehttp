#[cfg(feature = "json")]
use serde::Serialize;

#[cfg(feature = "multipart")]
use crate::multipart::MultipartBuilder;

/// Headers in a [`Request`] or [`Response`].
///
/// Note that the same header key can appear twice.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    /// Name-value pairs.
    pub headers: Vec<(String, String)>,
}

impl Headers {
    /// ```
    /// use ehttp::Request;
    /// let request = Request {
    ///     headers: ehttp::Headers::new(&[
    ///         ("Accept", "*/*"),
    ///         ("Content-Type", "text/plain; charset=utf-8"),
    ///     ]),
    ///     ..Request::get("https://www.example.com")
    /// };
    /// ```
    pub fn new(headers: &[(&str, &str)]) -> Self {
        Self {
            headers: headers
                .iter()
                .map(|e| (e.0.to_owned(), e.1.to_owned()))
                .collect(),
        }
    }

    /// Will add the key/value pair to the headers.
    ///
    /// If the key already exists, it will also be kept,
    /// so the same key can appear twice.
    pub fn insert(&mut self, key: impl ToString, value: impl ToString) {
        self.headers.push((key.to_string(), value.to_string()));
    }

    /// Get the value of the first header with the given key.
    ///
    /// The lookup is case-insensitive.
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_string().to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key)
            .map(|(_, v)| v.as_str())
    }

    /// Get all the values that match the given key.
    ///
    /// The lookup is case-insensitive.
    pub fn get_all(&self, key: &str) -> impl Iterator<Item = &str> {
        let key = key.to_string().to_lowercase();
        self.headers
            .iter()
            .filter(move |(k, _)| k.to_lowercase() == key)
            .map(|(_, v)| v.as_str())
    }

    /// Sort the headers by key.
    ///
    /// This makes the headers easier to read when printed out.
    ///
    /// `ehttp` will sort the headers in the responses.
    pub fn sort(&mut self) {
        self.headers.sort_by(|a, b| a.0.cmp(&b.0));
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.headers.into_iter()
    }
}

impl<'h> IntoIterator for &'h Headers {
    type Item = &'h (String, String);
    type IntoIter = std::slice::Iter<'h, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.headers.iter()
    }
}

// ----------------------------------------------------------------------------

/// Determine if cross-origin requests lead to valid responses.
/// Based on <https://developer.mozilla.org/en-US/docs/Web/API/Request/mode>
#[cfg(target_arch = "wasm32")]
#[derive(Default, Clone, Copy, Debug)]
pub enum Mode {
    /// If a request is made to another origin with this mode set, the result is an error.
    SameOrigin = 0,

    /// The request will not include the Origin header in a request.
    /// The server's response will be opaque, meaning that JavaScript code cannot access its contents
    NoCors = 1,

    /// Includes an Origin header in the request and expects the server to respond with an
    /// "Access-Control-Allow-Origin" header that indicates whether the request is allowed.
    #[default]
    Cors = 2,

    /// A mode for supporting navigation
    Navigate = 3,
}

#[cfg(target_arch = "wasm32")]
impl From<Mode> for web_sys::RequestMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::SameOrigin => web_sys::RequestMode::SameOrigin,
            Mode::NoCors => web_sys::RequestMode::NoCors,
            Mode::Cors => web_sys::RequestMode::Cors,
            Mode::Navigate => web_sys::RequestMode::Navigate,
        }
    }
}

/// A simple HTTP request.
#[derive(Clone, Debug)]
pub struct Request {
    /// "GET", "POST", …
    pub method: String,

    /// https://…
    pub url: String,

    /// The data you send with e.g. "POST".
    pub body: Vec<u8>,

    /// ("Accept", "*/*"), …
    pub headers: Headers,

    /// Request mode used on fetch. Only available on wasm builds
    #[cfg(target_arch = "wasm32")]
    pub mode: Mode,
}

impl Request {
    /// Create a `GET` request with the given url.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get(url: impl ToString) -> Self {
        Self {
            method: "GET".to_owned(),
            url: url.to_string(),
            body: vec![],
            headers: Headers::new(&[("Accept", "*/*")]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        }
    }

    /// Create a `HEAD` request with the given url.
    #[allow(clippy::needless_pass_by_value)]
    pub fn head(url: impl ToString) -> Self {
        Self {
            method: "HEAD".to_owned(),
            url: url.to_string(),
            body: vec![],
            headers: Headers::new(&[("Accept", "*/*")]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        }
    }

    /// Create a `POST` request with the given url and body.
    #[allow(clippy::needless_pass_by_value)]
    pub fn post(url: impl ToString, body: Vec<u8>) -> Self {
        Self {
            method: "POST".to_owned(),
            url: url.to_string(),
            body,
            headers: Headers::new(&[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        }
    }

    /// Multipart HTTP for both native and WASM.
    ///
    /// Requires the `multipart` feature to be enabled.
    ///
    /// Example:
    /// ```
    /// use std::io::Cursor;
    /// use ehttp::multipart::MultipartBuilder;
    /// let url = "https://www.example.com";
    /// let request = ehttp::Request::multipart(
    ///     url,
    ///     MultipartBuilder::new()
    ///         .add_text("label", "lorem ipsum")
    ///         .add_stream(
    ///             &mut Cursor::new(vec![0, 0, 0, 0]),
    ///             "4_empty_bytes",
    ///             Some("4_empty_bytes.png"),
    ///             None,
    ///         )
    ///         .unwrap(),
    /// );
    /// ehttp::fetch(request, |result| {});
    #[cfg(feature = "multipart")]
    pub fn multipart(url: impl ToString, builder: MultipartBuilder) -> Self {
        let (content_type, data) = builder.finish();
        Self {
            method: "POST".to_string(),
            url: url.to_string(),
            body: data,
            headers: Headers::new(&[("Accept", "*/*"), ("Content-Type", content_type.as_str())]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        }
    }

    #[cfg(feature = "json")]
    /// Create a `POST` request with the given url and json body.
    #[allow(clippy::needless_pass_by_value)]
    pub fn json<T>(url: impl ToString, body: &T) -> serde_json::error::Result<Self>
    where
        T: ?Sized + Serialize,
    {
        Ok(Self {
            method: "POST".to_owned(),
            url: url.to_string(),
            body: serde_json::to_string(body)?.into_bytes(),
            headers: Headers::new(&[("Accept", "*/*"), ("Content-Type", "application/json")]),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        })
    }
}

/// Response from a completed HTTP request.
#[derive(Clone)]
pub struct Response {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers.
    pub headers: Headers,

    /// The raw bytes of the response body.
    pub bytes: Vec<u8>,
}

impl Response {
    pub fn text(&self) -> Option<&str> {
        std::str::from_utf8(&self.bytes).ok()
    }

    /// Convenience for getting the `content-type` header.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            url,
            ok,
            status,
            status_text,
            headers,
            bytes,
        } = self;

        fmt.debug_struct("Response")
            .field("url", url)
            .field("ok", ok)
            .field("status", status)
            .field("status_text", status_text)
            .field("headers", headers)
            .field("bytes", &format!("{} bytes", bytes.len()))
            .finish_non_exhaustive()
    }
}

/// An HTTP response status line and headers used for the [`streaming`](crate::streaming) API.
#[derive(Clone, Debug)]
pub struct PartialResponse {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers.
    pub headers: Headers,
}

impl PartialResponse {
    pub fn complete(self, bytes: Vec<u8>) -> Response {
        let Self {
            url,
            ok,
            status,
            status_text,
            headers,
        } = self;
        Response {
            url,
            ok,
            status,
            status_text,
            headers,
            bytes,
        }
    }
}

/// A description of an error.
///
/// This is only used when we fail to make a request.
/// Any response results in `Ok`, including things like 404 (file not found).
pub type Error = String;

/// A type-alias for `Result<T, ehttp::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
