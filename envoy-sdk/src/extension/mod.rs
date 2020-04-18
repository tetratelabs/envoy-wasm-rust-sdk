extern crate std;
use std::prelude::v1::*;

pub mod access_logger;
pub mod filter;
pub mod factory;

pub use factory::Factory;

#[derive(Debug)]
pub enum Error {
    Host(proxy_wasm::types::Status),
    Extension,
}

impl From<proxy_wasm::types::Status> for Error {
    fn from(status: proxy_wasm::types::Status) -> Self {
        Error::Host(status)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
