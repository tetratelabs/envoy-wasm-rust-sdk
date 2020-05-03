extern crate std;
use std::fmt;
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

/// Opaque identifier of an extension instance.
pub struct InstanceId(u32);

impl From<u32> for InstanceId {
    fn from(context_id: u32) -> Self {
        InstanceId(context_id)
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
