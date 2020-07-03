# Examples

## List

* [http-filter](./http-filter) - logs HTTP request/response headers, makes an outgoing HTTP request
* [network-filter](./network-filter/) - logs start and end of a TCP conection, makes an outgoing HTTP request
* [access-logger](./access-logger) - logs information about an HTTP request or a TCP connection, makes an outgoing HTTP request

## How To

### How to Set up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm --release
```

### How to Run unit tests

```shell
cargo test
```
