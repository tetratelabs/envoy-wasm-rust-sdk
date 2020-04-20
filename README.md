# Rust SDK for Envoy Wasm extensions

Convenience layer on top of original [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) SDK
that brings in structure and guidance for extension developers.

## Components

* [envoy-sdk](./envoy-sdk/) - Envoy SDK
* [examples](./examples/) - Envoy SDK usage examples
  * [http-filter](./examples/http-filter/) - logs HTTP request/response headers, makes an outgoing HTTP request
  * [network-filter](./examples/network-filter/) - logs start and end of a TCP conection, makes an outgoing HTTP request
  * [access-logger](./examples/access-logger/) - logs information about an HTTP request or a TCP connection, makes an outgoing HTTP request
