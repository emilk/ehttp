#![cfg(not(target_arch = "wasm32"))]

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::time::Duration;

#[test]
fn decodes_gzip_without_a_tls_provider() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
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
