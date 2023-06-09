use std::collections::BTreeMap;

pub struct Response {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers. All header names are lower-case.
    pub headers: BTreeMap<String, String>,
}

pub enum Part {
    /// The header of the response.
    ///
    /// The `on_data` callback receives this only once.
    Response(Response),

    /// A single chunk of the response data.
    ///
    /// If the chunk is empty, that means the `on_data` callback will not receive any more data.
    Chunk(Vec<u8>),
}
