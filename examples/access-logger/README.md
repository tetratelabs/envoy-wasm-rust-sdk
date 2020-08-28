# Access Logger

Example of an Envoy Access Logger.

## How To

### How to Set up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm --release -p access-logger-module
```

### How To Run

```shell
docker-compose up
```

### How To Check

* Make an HTTP request
  ```shell
  curl -i localhost:10000/

  HTTP/1.1 200 OK
  content-length: 22
  content-type: text/plain
  date: Mon, 20 Apr 2020 17:54:07 GMT
  server: envoy
  x-envoy-upstream-service-time: 2

  Hi from mock service!
  ```

  ```shell
  wasm log examples.access_logger: logging at 2020-08-27T16:43:26.107301+00:00 with config: SampleAccessLoggerConfig { param: "value" }
  wasm log examples.access_logger:   request headers:
  wasm log examples.access_logger:     :authority: localhost:10000
  wasm log examples.access_logger:     :path: /
  wasm log examples.access_logger:     :method: GET
  wasm log examples.access_logger:     :scheme: http
  wasm log examples.access_logger:     user-agent: curl/7.64.1
  wasm log examples.access_logger:     accept: */*
  wasm log examples.access_logger:     x-forwarded-proto: http
  wasm log examples.access_logger:     x-request-id: 95104030-617a-4a9c-a0ea-66c5ab27b08d
  wasm log examples.access_logger:     x-envoy-expected-rq-timeout-ms: 15000
  wasm log examples.access_logger:   response headers:
  wasm log examples.access_logger:     :status: 200
  wasm log examples.access_logger:     content-length: 22
  wasm log examples.access_logger:     content-type: text/plain
  wasm log examples.access_logger:     date: Thu, 27 Aug 2020 16:43:26 GMT
  wasm log examples.access_logger:     server: envoy
  wasm log examples.access_logger:     x-envoy-upstream-service-time: 0
  wasm log examples.access_logger:   request info:
  wasm log examples.access_logger:     id: Some("95104030-617a-4a9c-a0ea-66c5ab27b08d")
  wasm log examples.access_logger:   connection info:
  wasm log examples.access_logger:     id: None
  wasm log examples.access_logger:   listener info:
  wasm log examples.access_logger:     traffic_direction: None
  wasm log examples.access_logger:   route info:
  wasm log examples.access_logger:     route.name: None
  wasm log examples.access_logger:   cluster info:
  wasm log examples.access_logger:     cluster.name: Some("mock_service")
  wasm log examples.access_logger:   upstream info:
  wasm log examples.access_logger:     address: Some("127.0.0.1:10001")
  wasm log examples.access_logger: sent request to a log collector: @2
  wasm log examples.access_logger: received response from a log collector on request: @2
  wasm log examples.access_logger:   headers[count=6]:
  wasm log examples.access_logger:     :status: 200
  wasm log examples.access_logger:     content-length: 22
  wasm log examples.access_logger:     content-type: text/plain
  wasm log examples.access_logger:     date: Thu, 27 Aug 2020 16:43:26 GMT
  wasm log examples.access_logger:     server: envoy
  wasm log examples.access_logger:     x-envoy-upstream-service-time: 1
  ```

### How to Run unit tests

```shell
cargo test
```
