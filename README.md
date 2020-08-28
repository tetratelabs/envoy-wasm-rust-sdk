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
envoy-sdk = "0.1"
```

## Components

* [envoy-sdk](./envoy-sdk/) - `Envoy SDK`
* [examples](./examples/) - `Envoy SDK` usage examples
  * [http-filter](./examples/http-filter/) - logs HTTP request/response headers, makes an outgoing HTTP request
  * [network-filter](./examples/network-filter/) - logs start and end of a TCP conection, makes an outgoing HTTP request
  * [access-logger](./examples/access-logger/) - logs information about an HTTP request or a TCP connection, makes an outgoing HTTP request
