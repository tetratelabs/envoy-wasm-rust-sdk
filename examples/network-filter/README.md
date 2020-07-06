# Network Filter

Example of an Envoy Network filter.

## How To

### How to Set up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm --release -p network-filter-module
```

### How To Run

```shell
docker-compose up
```

### How To Check

* Make a request that the HTTP filter will suspend in order to make an authorization request first:
  ```shell
  curl -i localhost:10000

  <pending>
  ```

  Check Envoy logs:
  ```shell
  wasm log envoy-sdk-samples network-filter: #2 new TCP connection starts at 2020-04-20T17:42:42.524006+00:00 with config: {"setting":"value"}
  wasm log envoy-sdk-samples network-filter: #2 sent outgoing request: @1
  wasm log envoy-sdk-samples network-filter: #2 received response on outgoing request: @1
  wasm log envoy-sdk-samples network-filter:      headers[count=6]:
  ```

  Notice that example get's stuck indefinitely because [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) misses some "resume" API.

### Known issues

* [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) doesn't
  support resume operation for TCP streams

### How to Run unit tests

```shell
cargo test
```
