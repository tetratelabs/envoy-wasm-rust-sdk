# Http Filter

Example of an Envoy HTTP filter.

## How To

### How to Set up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm --release -p http-filter-module
```

### How To Run

```shell
docker-compose up
```

### How To Check

* Make a request that the HTTP filter will simply observe:
  ```shell
  curl -i localhost:10000

  HTTP/1.1 200 OK
  content-length: 22
  content-type: text/plain
  date: Mon, 20 Apr 2020 17:10:09 GMT
  server: envoy
  x-envoy-upstream-service-time: 5

  Hi from mock service!
  ```

  Check Envoy logs:
  ```shell
  wasm log envoy-sdk-samples http-filter: #2 observing request headers
  wasm log envoy-sdk-samples http-filter: #2 -> :authority: localhost:10000
  wasm log envoy-sdk-samples http-filter: #2 -> :path: /secret
  wasm log envoy-sdk-samples http-filter: #2 -> :method: GET
  wasm log envoy-sdk-samples http-filter: #2 -> user-agent: curl/7.64.1
  wasm log envoy-sdk-samples http-filter: #2 -> accept: */*
  wasm log envoy-sdk-samples http-filter: #2 -> x-forwarded-proto: http
  wasm log envoy-sdk-samples http-filter: #2 -> x-request-id: adf98512-8e20-4541-8263-fbc509869e63
  wasm log envoy-sdk-samples http-filter: #2 observing response headers
  wasm log envoy-sdk-samples http-filter: #2 <- :status: 200
  wasm log envoy-sdk-samples http-filter: #2 <- content-length: 22
  wasm log envoy-sdk-samples http-filter: #2 <- content-type: text/plain
  wasm log envoy-sdk-samples http-filter: #2 <- date: Mon, 20 Apr 2020 17:24:44 GMT
  wasm log envoy-sdk-samples http-filter: #2 <- server: envoy
  wasm log envoy-sdk-samples http-filter: #2 <- x-envoy-upstream-service-time: 0
  wasm log envoy-sdk-samples http-filter: #2 http exchange complete
  ```

* Make a request that the HTTP filter will respond to by itself (without passing to the upstream):
  ```shell
  curl -i localhost:10000/ping

  HTTP/1.1 200 OK
  content-length: 6
  content-type: text/plain
  x-sample-response: pong
  date: Mon, 20 Apr 2020 17:15:19 GMT
  server: envoy

  Pong!
  ```

  Notice that this time the response is from the HTTP filter itself.

* Make a request that the HTTP filter will suspend in order to make an authorization request first:
  ```shell
  curl -i localhost:10000/secret

  HTTP/1.1 200 OK
  content-length: 22
  content-type: text/plain
  date: Mon, 20 Apr 2020 17:31:02 GMT
  server: envoy
  x-envoy-upstream-service-time: 0

  Hi from mock service!
  ```

  Check Envoy logs:
  ```shell
  wasm log envoy-sdk-samples http-filter: #2 observing request headers
  wasm log envoy-sdk-samples http-filter: #2 -> :authority: localhost:10000
  wasm log envoy-sdk-samples http-filter: #2 -> :path: /secret
  wasm log envoy-sdk-samples http-filter: #2 -> :method: GET
  wasm log envoy-sdk-samples http-filter: #2 -> user-agent: curl/7.64.1
  wasm log envoy-sdk-samples http-filter: #2 -> accept: */*
  wasm log envoy-sdk-samples http-filter: #2 -> x-forwarded-proto: http
  wasm log envoy-sdk-samples http-filter: #2 -> x-request-id: adf98512-8e20-4541-8263-fbc509869e63
  wasm log envoy-sdk-samples http-filter: #2 sent authorization request: @1
  wasm log envoy-sdk-samples http-filter: #2 suspending http exchange processing
  wasm log envoy-sdk-samples http-filter: #2 received response on authorization request: @1
  wasm log envoy-sdk-samples http-filter:      headers[count=6]:
  wasm log envoy-sdk-samples http-filter: #2 resuming http exchange processing
  wasm log envoy-sdk-samples http-filter: #2 observing response headers
  wasm log envoy-sdk-samples http-filter: #2 <- :status: 200
  wasm log envoy-sdk-samples http-filter: #2 <- content-length: 22
  wasm log envoy-sdk-samples http-filter: #2 <- content-type: text/plain
  wasm log envoy-sdk-samples http-filter: #2 <- date: Mon, 20 Apr 2020 17:24:44 GMT
  wasm log envoy-sdk-samples http-filter: #2 <- server: envoy
  wasm log envoy-sdk-samples http-filter: #2 <- x-envoy-upstream-service-time: 0
  wasm log envoy-sdk-samples http-filter: #2 http exchange complete
  ```

  Notice that request processing was suspended until authorization request was complete.

  ### Known issues

* [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) doesn't support configuration of Http Filters
* [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) has an issue that doesn't allow to access headers of the response to outgoing HTTP request

### How to Run unit tests

Run tests:
```
cargo test
```
