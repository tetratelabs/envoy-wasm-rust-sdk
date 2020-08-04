[![Build](https://github.com/tetratelabs/envoy-wasm-rust-sdk/workflows/build/badge.svg)](https://github.com/tetratelabs/envoy-wasm-rust-sdk/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

# Rust SDK for WebAssembly-based Envoy extensions

Convenience layer on top of the original [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) SDK
that brings in structure and guidance for extension developers.

## Components

* [src/](./src/)
  * [extension/](./src/extension/) - base types for various Envoy extensions
    * [access_logger/](./src/extension/access_logger/) - base types for Envoy Access Loggers
    * [filter/](./src/extension/filter/) - base types for Envoy filters
      * [http/](./src/extension/filter/http/) - base types for Envoy HTTP filters
      * [network/](./src/extension/filter/network/) - base types for Envoy Network filters
  * [host/](./src/host/) - types to represent various Envoy APIs
    * [services/](./src/host/services/) - types to represent various Envoy services available for use by extensions
      * [time.rs](./src/host/services/time.rs) - Time service
      * etc

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
