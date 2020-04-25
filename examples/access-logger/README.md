# Access Logger

Example of an Envoy Access Logger.

## How To

### How to Set-up Rust

```shell
rustup target add wasm32-unknown-unknown
```

### How To Build

```shell
cargo build:wasm --release
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
  wasm log access-logger: logging at 2020-04-20T17:54:08.268442+00:00 with config: config for access-logger
  wasm log access-logger:   request headers:
  wasm log access-logger:     :authority: localhost:10000
  wasm log access-logger:     :path: /
  wasm log access-logger:     :method: GET
  wasm log access-logger:     :scheme: http
  wasm log access-logger:     user-agent: curl/7.64.1
  wasm log access-logger:     accept: */*
  wasm log access-logger:     x-forwarded-proto: http
  wasm log access-logger:     x-request-id: 216ae30c-9ed0-42d4-993d-0094c42acea2
  wasm log access-logger:     x-envoy-expected-rq-timeout-ms: 15000
  wasm log access-logger:   response headers:
  wasm log access-logger:     :status: 200
  wasm log access-logger:     content-length: 22
  wasm log access-logger:     content-type: text/plain
  wasm log access-logger:     date: Mon, 20 Apr 2020 17:54:07 GMT
  wasm log access-logger:     server: envoy
  wasm log access-logger:     x-envoy-upstream-service-time: 2
  wasm log access-logger:   upstream info:
  wasm log access-logger:     upstream.address: 127.0.0.1:10001
  wasm log access-logger: sent request to a log collector: @1
  ```
* Make N more HTTP requests (where N is a number of Envoy worker threads)
  ```shell
  curl -i localhost:10000/

  curl: (56) Recv failure: Connection reset by peer
  ```

```shell
envoy_1  | [2020-04-20 18:07:13.595][20][critical][wasm] [source/extensions/common/wasm/context.cc:1101] wasm log access-logger: panicked at 'invalid context_id', /home/builder/.cargo/registry/src/github.com-1ecc6299db9ec823/proxy-wasm-0.1.0/src/dispatcher.rs:183:13
envoy_1  | [2020-04-20 18:07:13.602][20][critical][main] [source/exe/terminate_handler.cc:13] std::terminate called! (possible uncaught exception, see trace)
envoy_1  | [2020-04-20 18:07:13.602][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:91] Backtrace (use tools/stack_decode.py to get line numbers):
envoy_1  | [2020-04-20 18:07:13.602][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:92] Envoy version: d29f7a659ba736aab97697a7bcfc69a71bc66b66/1.14.0-dev/Clean/RELEASE/BoringSSL
envoy_1  | [2020-04-20 18:07:13.614][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:96] #0: Envoy::TerminateHandler::logOnTerminate()::$_0::operator()() [0x55ea34500bbe]
envoy_1  | [2020-04-20 18:07:13.625][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:98] #1: [0x55ea34500ac9]
envoy_1  | [2020-04-20 18:07:13.639][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:96] #2: std::__terminate() [0x55ea34a87893]
envoy_1  | [2020-04-20 18:07:13.653][20][critical][backtrace] [bazel-out/k8-opt/bin/source/server/_virtual_includes/backtrace_lib/server/backtrace.h:96] #3: Envoy::Http::ConnectionManagerImpl::ActiveStream::~ActiveStream() [0x55ea3429e815]
```

### Known issues:

* [proxy-wasm](https://github.com/proxy-wasm/proxy-wasm-rust-sdk) destroys Access Logger after the first use.
  Any subsequent request (to ther same Envoy worker thread) will crash Envoy because Access Logger has already been destroyed.
* in the attached log one can notice that Access Logger never receives a response on the request to a log collector.
  This is caused by the issue above.

### How to Run unit tests

One-off setup:
```shell
rustup target add wasm32-wasi
cargo install cargo-wasi
curl https://wasmtime.dev/install.sh -sSf | bash
```

Run tests:
```
cargo wasi test --no-default-features
```
