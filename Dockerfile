#
# Builder image
#

FROM rust:1.42

RUN rustup target add wasm32-unknown-unknown
