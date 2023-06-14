use crate::types::PartialResponse;

/// A piece streamed by [`crate::streaming::fetch`].
pub enum Part {
    /// The header of the response.
    ///
    /// The `on_data` callback receives this only once.
    Response(PartialResponse),

    /// A single chunk of the response data.
    ///
    /// If the chunk is empty, that means the `on_data` callback will not receive any more data.
    Chunk(Vec<u8>),
}
