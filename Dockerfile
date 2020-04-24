#
# Builder image
#

FROM rust:1.42

RUN rustup target add wasm32-unknown-unknown

RUN rustup target add wasm32-wasi
RUN cargo install cargo-wasi
RUN curl https://wasmtime.dev/install.sh -sSf | bash
