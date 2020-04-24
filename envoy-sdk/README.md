# # Rust SDK for Envoy Wasm extensions

Convenience layer on top of original [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) SDK
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

### How to Set-up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build --release
```

### How to Run unit tests

One-off setup:
```shell
rustup target add wasm32-wasi
cargo install cargo-wasi
curl https://wasmtime.dev/install.sh -sSf | bash
```

Run tests:
```
cargo wasi test
```
