#![cfg(not(target_arch = "wasm32"))]

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::time::Duration;

#[test]
fn decodes_gzip() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        while line != "\r\n" {
            line.clear();
            reader.read_line(&mut line).unwrap();
        }

        // A gzip-compressed "hello" checks that disabling Ureq's default features
        // does not also disable transparent response decompression.
        let body = [
            0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xcb, 0x48, 0xcd, 0xc9,
            0xc9, 0x07, 0x00, 0x86, 0xa6, 0x10, 0x36, 0x05, 0x00, 0x00, 0x00,
        ];
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(&body).unwrap();
    });

    let response = ehttp::fetch_blocking(&ehttp::Request::get(format!("http://{address}")));
    server.join().unwrap();
    let response = response.unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.bytes, b"hello");
}

#[cfg(not(feature = "rustls-ring"))]
#[test]
fn https_without_a_tls_provider_is_an_error() {
    let result = ehttp::fetch_blocking(&ehttp::Request::get("HTTPS://127.0.0.1:1"));
    let err = result.err().unwrap_or_default();
    assert!(err.contains("CryptoProvider"), "{}", err);
}
