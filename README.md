# ehttp: a minimal Rust HTTP client for both native and WASM

[![Latest version](https://img.shields.io/crates/v/ehttp.svg)](https://crates.io/crates/ehttp)
[![Documentation](https://docs.rs/ehttp/badge.svg)](https://docs.rs/ehttp)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Build Status](https://github.com/emilk/ehttp/workflows/CI/badge.svg)](https://github.com/emilk/ehttp/actions?workflow=CI)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

If you want to do HTTP requests and are targeting both native and web (WASM), then this is the crate for you!

[You can try the web demo here](https://emilk.github.io/ehttp/index.html) (works in any browser with WASM and WebGL support). Uses [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe).

## Usage
``` rust
let request = ehttp::Request::get("https://www.example.com");
ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
    println!("Status code: {:?}", result.unwrap().status);
});
```

The given callback is called when the request is completed.
You can communicate the results back to the main thread using something like:

* Channels (e.g. [`std::sync::mpsc::channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html)).
* `Arc<Mutex<_>>`
* [`poll_promise::Promise`](https://docs.rs/poll-promise)
* [`eventuals::Eventual`](https://docs.rs/eventuals/latest/eventuals/struct.Eventual.html)
* [`tokio::sync::watch::channel`](https://docs.rs/tokio/latest/tokio/sync/watch/fn.channel.html)

There is also a streaming version under `ehttp::fetch::streaming`, hidden behind the `streaming` feature flag.
