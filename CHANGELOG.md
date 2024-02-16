# egui changelog

All notable changes to the `ehttp` crate will be documented in this file.


## Unreleased


## 0.5.0 - 2024-02-16
* Support multipart and JSON ([#47](https://github.com/emilk/ehttp/pull/47), [#49](https://github.com/emilk/ehttp/pull/49))
* Added CORS `Mode` property to `Request` on web ([#52](https://github.com/emilk/ehttp/pull/52))
* Don't add `web-sys` in native builds ([#48](https://github.com/emilk/ehttp/pull/48))


## 0.4.0 - 2024-01-17
* Allow duplicated headers in requests and responses ([#46](https://github.com/emilk/ehttp/pull/46))
* Support HEAD requests ([#45](https://github.com/emilk/ehttp/pull/45))
* Add missing web-sys feature ([#42](https://github.com/emilk/ehttp/pull/42))
* Update MSRV to 1.72.0 ([#44](https://github.com/emilk/ehttp/pull/44))


## 0.3.1 - 2023-09-27
* Improve opaque network error message on web ([#33](https://github.com/emilk/ehttp/pull/33)).


## 0.3.0 - 2023-06-15
* Add `ehttp::streaming`, for streaming HTTP requests ([#28](https://github.com/emilk/ehttp/pull/28)).
* Add cross-platform `fetch_async` ([#25](https://github.com/emilk/ehttp/pull/25)).
* Nicer formatted error messages on web.
* Implement `Clone` and `Debug` for `Request` ([#17](https://github.com/emilk/ehttp/pull/17)).


## 0.2.0 - 2022-01-15
* `Response::text` and `Response::content_type` no longer allocates.
* Rename `ehttp::Request::create_headers_map` to `ehttp::headers`.
* `Request::post` now expects `Vec<u8>`.


## 0.1.0 - 2021-09-03 - First release
