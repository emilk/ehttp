use crate::types::PartialResponse;

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

pub enum Control {
    Continue,
    Break,
}

impl Control {
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break)
    }

    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}
