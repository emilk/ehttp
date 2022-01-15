# egui changelog

All notable changes to the `ehttp` crate will be documented in this file.


## Unreleased
* `Response::text` and `Response::content_type` no longer allocates.
* Rename `ehttp::Request::create_headers_map` to `ehttp::headers`.
* `Request::post` now expects `Vec<u8>`.


## 0.1.0 - 2021-09-03 - First release
