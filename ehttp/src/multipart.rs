//! Multipart HTTP request for both native and WASM.
//!
//! Requires the `multipart` feature to be enabled.
//!
//! Example:
//! ```
//! use std::io::Cursor;
//! use ehttp::multipart::MultipartBuilder;
//! let url = "https://www.example.com";
//! let request = ehttp::Request::multipart(
//!     url,
//!     MultipartBuilder::new()
//!         .add_text("label", "lorem ipsum")
//!         .add_stream(
//!             &mut Cursor::new(vec![0, 0, 0, 0]),
//!             "4_empty_bytes",
//!             Some("4_empty_bytes.png"),
//!             None,
//!         )
//!         .unwrap(),
//! );
//! ehttp::fetch(request, |result| {});
//! ```
//! Taken from ureq_multipart 1.1.1
//!

use mime::Mime;
use rand::Rng;

use std::io::{self, Read, Write};

const BOUNDARY_LEN: usize = 29;

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::from(0..=9))
        .take(len)
        .map(|num| num.to_string())
        .collect()
}

#[derive(Debug)]
/// The Builder for the multipart
pub struct MultipartBuilder {
    boundary: String,
    inner: Vec<u8>,
    data_written: bool,
}

impl Default for MultipartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl MultipartBuilder {
    /// creates a new MultipartBuilder with empty inner
    pub fn new() -> Self {
        Self {
            boundary: random_alphanumeric(BOUNDARY_LEN),
            inner: Vec::new(),
            data_written: false,
        }
    }

    /// add text field
    ///
    /// * name field name
    /// * text field text value
    pub fn add_text(mut self, name: &str, text: &str) -> Self {
        self.write_field_headers(name, None, None);
        self.inner.extend(text.as_bytes());
        self
    }

    /// add file
    ///
    /// * name file field name
    /// * path the sending file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn add_file<P: AsRef<std::path::Path>>(self, name: &str, path: P) -> io::Result<Self> {
        fn mime_filename(path: &std::path::Path) -> (Mime, Option<&str>) {
            let content_type = mime_guess::from_path(path);
            let filename = path.file_name().and_then(|filename| filename.to_str());
            (content_type.first_or_octet_stream(), filename)
        }

        let path = path.as_ref();
        let (content_type, filename) = mime_filename(path);
        let mut file = std::fs::File::open(path)?;
        self.add_stream(&mut file, name, filename, Some(content_type))
    }

    /// add some stream
    pub fn add_stream<S: Read>(
        mut self,
        stream: &mut S,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) -> io::Result<Self> {
        // This is necessary to make sure it is interpreted as a file on the server end.
        let content_type = Some(content_type.unwrap_or(mime::APPLICATION_OCTET_STREAM));
        self.write_field_headers(name, filename, content_type);
        io::copy(stream, &mut self.inner)?;
        Ok(self)
    }

    fn write_boundary(&mut self) {
        if self.data_written {
            self.inner.write_all(b"\r\n").unwrap();
        }

        write!(
            self.inner,
            "-----------------------------{}\r\n",
            self.boundary
        )
        .unwrap()
    }

    fn write_field_headers(
        &mut self,
        name: &str,
        filename: Option<&str>,
        content_type: Option<Mime>,
    ) {
        self.write_boundary();
        if !self.data_written {
            self.data_written = true;
        }
        write!(
            self.inner,
            "Content-Disposition: form-data; name=\"{name}\""
        )
        .unwrap();
        if let Some(filename) = filename {
            write!(self.inner, "; filename=\"{filename}\"").unwrap();
        }
        if let Some(content_type) = content_type {
            write!(self.inner, "\r\nContent-Type: {content_type}").unwrap();
        }
        self.inner.write_all(b"\r\n\r\n").unwrap();
    }

    /// general multipart data
    ///
    /// # Return
    /// * (content_type,post_data)
    ///    * content_type http header content type
    ///    * post_data ureq.req.send_send_bytes(&post_data)
    ///
    pub fn finish(mut self) -> (String, Vec<u8>) {
        if self.data_written {
            self.inner.write_all(b"\r\n").unwrap();
        }

        // always write the closing boundary, even for empty bodies
        write!(
            self.inner,
            "-----------------------------{}--\r\n",
            self.boundary
        )
        .unwrap();
        (
            format!(
                "multipart/form-data; boundary=---------------------------{}",
                self.boundary
            ),
            self.inner,
        )
    }
}
