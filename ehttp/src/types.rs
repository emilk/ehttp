use std::time::Duration;

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

impl<const N: usize> From<&[(&str, &str); N]> for Headers {
    fn from(headers: &[(&str, &str); N]) -> Self {
        Self::new(headers.as_slice())
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
///
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

// ----------------------------------------------------------------------------

/// Determines whether or not the browser sends credentials with the request, as well as whether any Set-Cookie response headers are respected.
///
/// Based on <https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials>
#[cfg(target_arch = "wasm32")]
#[derive(Default, Clone, Copy, Debug)]
pub enum Credentials {
    /// Never send credentials in the request or include credentials in the response.
    #[default]
    Omit = 0,

    /// Only send and include credentials for same-origin requests.
    SameOrigin = 1,

    /// Always include credentials, even for cross-origin requests.
    Include = 2,
}

#[cfg(target_arch = "wasm32")]
impl From<Credentials> for web_sys::RequestCredentials {
    fn from(credentials: Credentials) -> Self {
        match credentials {
            Credentials::Omit => web_sys::RequestCredentials::Omit,
            Credentials::SameOrigin => web_sys::RequestCredentials::SameOrigin,
            Credentials::Include => web_sys::RequestCredentials::Include,
        }
    }
}

/// A simple HTTP request.
#[derive(Clone, Debug)]
pub struct Request {
    /// "GET", "POST", …
    pub method: Method,

    /// https://…
    pub url: String,

    /// The data you send with e.g. "POST".
    pub body: Vec<u8>,

    /// ("Accept", "*/*"), …
    pub headers: Headers,

    /// Cancel the request if it doesn't complete fast enough.
    pub timeout: Option<Duration>,

    /// Request mode used on fetch.
    ///
    /// Used on Web to control CORS.
    #[cfg(target_arch = "wasm32")]
    pub mode: Mode,

    /// Credential options for fetch.
    ///
    /// Only applies to the web backend.
    #[cfg(target_arch = "wasm32")]
    pub credentials: Credentials,
}

impl Request {
    /// The default timeout for requests (30 seconds).
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

    /// Create a new request with the given method, url, and headers.
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(method: Method, url: impl ToString, headers: impl Into<Headers>) -> Self {
        Self {
            method,
            url: url.to_string(),
            body: vec![],
            headers: headers.into(),
            timeout: Some(Self::DEFAULT_TIMEOUT),
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
            #[cfg(target_arch = "wasm32")]
            credentials: Credentials::default(),
        }
    }

    /// Create a `GET` request with the given url.
    pub fn get(url: impl ToString) -> Self {
        Self::new(Method::GET, url, &[("Accept", "*/*")])
    }

    /// Create a `HEAD` request with the given url.
    pub fn head(url: impl ToString) -> Self {
        Self::new(Method::HEAD, url, &[("Accept", "*/*")])
    }

    /// Create a `POST` request with the given url and body.
    pub fn post(url: impl ToString, body: Vec<u8>) -> Self {
        Self::new(
            Method::POST,
            url,
            &[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ],
        )
        .with_body(body)
    }

    /// Create a 'PUT' request with the given url and body.
    pub fn put(url: impl ToString, body: Vec<u8>) -> Self {
        Self::new(
            Method::PUT,
            url,
            &[
                ("Accept", "*/*"),
                ("Content-Type", "text/plain; charset=utf-8"),
            ],
        )
        .with_body(body)
    }

    /// Create a 'DELETE' request with the given url.
    pub fn delete(url: &str) -> Self {
        Self::new(Method::DELETE, url, &[("Accept", "*/*")])
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
    /// let request = ehttp::Request::post_multipart(
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
    /// ```
    #[cfg(feature = "multipart")]
    pub fn post_multipart(url: impl ToString, builder: MultipartBuilder) -> Self {
        let (content_type, data) = builder.finish();
        Self::new(
            Method::POST,
            url,
            Headers::new(&[("Accept", "*/*"), ("Content-Type", content_type.as_str())]),
        )
        .with_body(data)
    }

    #[cfg(feature = "multipart")]
    #[deprecated(note = "Renamed to `post_multipart`")]
    pub fn multipart(url: impl ToString, builder: MultipartBuilder) -> Self {
        Self::post_multipart(url, builder)
    }

    #[cfg(feature = "json")]
    /// Create a `POST` request with the given url and json body.
    pub fn post_json<T>(url: impl ToString, body: &T) -> serde_json::error::Result<Self>
    where
        T: ?Sized + Serialize,
    {
        Ok(Self::new(
            Method::POST,
            url,
            &[("Accept", "*/*"), ("Content-Type", "application/json")],
        )
        .with_body(serde_json::to_string(body)?.into_bytes()))
    }

    #[cfg(feature = "json")]
    #[deprecated(note = "Renamed to `post_json`")]
    pub fn json<T>(url: impl ToString, body: &T) -> serde_json::error::Result<Self>
    where
        T: ?Sized + Serialize,
    {
        Self::post_json(url, body)
    }

    #[cfg(feature = "json")]
    /// Create a 'PUT' request with the given url and json body.
    pub fn put_json<T>(url: impl ToString, body: &T) -> serde_json::error::Result<Self>
    where
        T: ?Sized + Serialize,
    {
        Ok(Self::new(
            Method::PUT,
            url,
            &[("Accept", "*/*"), ("Content-Type", "application/json")],
        )
        .with_body(serde_json::to_string(body)?.into_bytes()))
    }

    /// Set the HTTP method.
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the URL.
    pub fn with_url(mut self, url: impl ToString) -> Self {
        self.url = url.to_string();
        self
    }

    /// Set the request body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Replace all headers.
    pub fn with_headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Append a single header to the request.
    pub fn with_header(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set the request timeout, or `None` to disable it.
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the request mode (controls CORS behavior on web).
    #[cfg(target_arch = "wasm32")]
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Set whether credentials are sent with the request (web only).
    #[cfg(target_arch = "wasm32")]
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = credentials;
        self
    }

    /// Fetch the ureq response from a page
    #[cfg(not(target_arch = "wasm32"))]
    pub fn fetch_raw_native(&self, with_timeout: bool) -> Result<ureq::http::Response<ureq::Body>> {
        if self.method.contains_body() {
            let mut req = match self.method {
                Method::POST => ureq::post(&self.url),
                Method::PATCH => ureq::patch(&self.url),
                Method::PUT => ureq::put(&self.url),
                // These three are the only requests which contain a body, no other requests will be matched
                _ => unreachable!(), // because of the `.contains_body()` call
            };

            for (k, v) in &self.headers {
                req = req.header(k, v);
            }

            req = {
                if with_timeout {
                    req.config()
                } else {
                    req.config().timeout_recv_body(self.timeout)
                }
                .http_status_as_error(false)
                .build()
            };

            if self.body.is_empty() {
                req.send_empty()
            } else {
                req.send(&self.body)
            }
        } else {
            let mut req = match self.method {
                Method::GET => ureq::get(&self.url),
                Method::DELETE => ureq::delete(&self.url),
                Method::CONNECT => ureq::connect(&self.url),
                Method::HEAD => ureq::head(&self.url),
                Method::OPTIONS => ureq::options(&self.url),
                Method::TRACE => ureq::trace(&self.url),
                // Include all other variants rather than a catch all here to prevent confusion if another variant were to be added
                Method::PATCH | Method::POST | Method::PUT => unreachable!(), // because of the `.contains_body()` call
            };

            req = req
                .config()
                .timeout_recv_body(self.timeout)
                .http_status_as_error(false)
                .build();

            for (k, v) in &self.headers {
                req = req.header(k, v);
            }

            if self.body.is_empty() {
                req.call()
            } else {
                req.force_send_body().send(&self.body)
            }
        }
        .map_err(|err| err.to_string())
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

    #[cfg(feature = "json")]
    /// Convenience for getting json body
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(self.bytes.as_slice())
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

/// An [HTTP method](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Methods)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl Method {
    /// Whether ureq creates a `RequestBuilder<WithBody>` or `RequestBuilder<WithoutBody>`
    pub fn contains_body(&self) -> bool {
        use Method::*;
        match self {
            // Methods that are created with a body
            POST | PATCH | PUT => true,
            // Everything else
            _ => false,
        }
    }

    /// Convert an HTTP method string ("GET", "HEAD") to its enum variant
    pub fn parse(string: &str) -> Result<Self> {
        use Method::*;
        match string {
            "GET" => Ok(GET),
            "HEAD" => Ok(HEAD),
            "POST" => Ok(POST),
            "PUT" => Ok(PUT),
            "DELETE" => Ok(DELETE),
            "CONNECT" => Ok(CONNECT),
            "OPTIONS" => Ok(OPTIONS),
            "TRACE" => Ok(TRACE),
            "PATCH" => Ok(PATCH),
            _ => Err(Error::from("Failed to parse HTTP method")),
        }
    }

    pub fn as_str(&self) -> &'static str {
        use Method::*;
        match self {
            GET => "GET",
            HEAD => "HEAD",
            POST => "POST",
            PUT => "PUT",
            DELETE => "DELETE",
            CONNECT => "CONNECT",
            OPTIONS => "OPTIONS",
            TRACE => "TRACE",
            PATCH => "PATCH",
        }
    }
}
