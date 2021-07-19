[![Build](https://github.com/tetratelabs/envoy-wasm-rust-sdk/workflows/build/badge.svg)](https://github.com/tetratelabs/envoy-wasm-rust-sdk/actions)
[![Crate](https://img.shields.io/crates/v/envoy-sdk.svg)](https://crates.io/crates/envoy-sdk)
[![Docs](https://docs.rs/envoy-sdk/badge.svg)](https://docs.rs/envoy-sdk)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

# Rust SDK for WebAssembly-based Envoy extensions

Convenience layer on top of the original [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) SDK
that brings in structure and guidance for extension developers.

## Components

* [src/](./src/)
  * [extension/](./src/extension/) - base types for various `Envoy` extensions
    * [access_logger/](./src/extension/access_logger/) - base types for `Envoy` `Access Logger`s
    * [filter/](./src/extension/filter/) - base types for `Envoy` filters
      * [http/](./src/extension/filter/http/) - base types for `Envoy` `HTTP filters`
  * [host/](./src/host/) - types to represent various `Envoy APIs`
    * [http/](./src/host/http/client.rs) - `Envoy` `HTTP Client API`
    * [stream_info/](./src/host/stream_info/mod.rs) - `Envoy` `Stream Info API`
    * [log](./src/host/log.rs) - `Envoy` `Log API`
    * [shared_data](./src/host/shared_data.rs) - `Envoy` `Shared Data API`
    * [shared_queue](./src/host/shared_queue.rs) - `Envoy` `Shared Queue API`
    * [stats](./src/host/stats.rs) - `Envoy` `Stats API`
    * [time](./src/host/time.rs) - `Envoy` `Time API`

## How To

### How to Set up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm
```

### How to Run unit tests

```shell
cargo test
```
