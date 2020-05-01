extern crate std;
use std::prelude::v1::*;

pub mod access_logger;
pub mod factory;
pub mod filter;

pub use factory::Factory;

#[derive(Debug)]
pub enum Error {
    HostCall(&'static str, proxy_wasm::types::Status),
    Extension,
}

impl From<(&'static str, proxy_wasm::types::Status)> for Error {
    fn from(pair: (&'static str, proxy_wasm::types::Status)) -> Self {
        Error::HostCall(pair.0, pair.1)
    }
}

/// The type returned by extension methods.
pub type Result<T> = core::result::Result<T, Error>;
