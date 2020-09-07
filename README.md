[![Build](https://github.com/tetratelabs/envoy-wasm-rust-sdk/workflows/build/badge.svg)](https://github.com/tetratelabs/envoy-wasm-rust-sdk/actions)
[![Crate](https://img.shields.io/crates/v/envoy-sdk.svg)](https://crates.io/crates/envoy-sdk)
[![Docs](https://docs.rs/envoy-sdk/badge.svg)](https://docs.rs/envoy-sdk)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

# Rust SDK for WebAssembly-based Envoy extensions

Convenience layer on top of the original [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) SDK
that brings in structure and guidance for extension developers.

## TLDR

```toml
[dependencies]
envoy = { package = "envoy-sdk", version = "0.1" }
```

```rust
use envoy::extension::filter::http;
use envoy::extension::{HttpFilter, Result};
use envoy::host::log;

/// My very own `HttpFilter`.
struct MyHttpFilter;

impl HttpFilter for MyHttpFilter {
    /// Called when HTTP request headers have been received.
    ///
    /// Use `_ops` to access or mutate request headers.
    fn on_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool, _ops: &dyn http::RequestHeadersOps) -> Result<http::FilterHeadersStatus> {
        log::info!("proxying an HTTP request");
        Ok(http::FilterHeadersStatus::Continue)
    }
}
```

## Components

* [envoy-sdk](./envoy-sdk/) - `Envoy SDK`
* [envoy-sdk-test](./envoy-sdk-test/) - `Unit Test Framework` accompanying `Envoy SDK`
* [examples](./examples/) - `Envoy SDK` usage examples
  * [http-filter](./examples/http-filter/) - logs HTTP request/response headers, makes an outgoing HTTP request
  * [network-filter](./examples/network-filter/) - logs start and end of a TCP conection, makes an outgoing HTTP request
  * [access-logger](./examples/access-logger/) - logs information about an HTTP request or a TCP connection, makes an outgoing HTTP request

## Latest docs (on `master`)

* [envoy-sdk](https://tetratelabs.github.io/envoy-wasm-rust-sdk/envoy_sdk/)
* [envoy-sdk-test](https://tetratelabs.github.io/envoy-wasm-rust-sdk/envoy_sdk_test/)
