extern crate std;
use std::prelude::v1::*;

pub mod ops;
pub mod context;
pub use context::LoggerContext; 

use crate::host;
use crate::extension::Result;

use proxy_wasm::types::Bytes;

pub trait Logger {
    fn on_log(&mut self, _ops: &dyn LogOps) -> Result<()> {
        Ok(())
    }
}

pub trait LogOps {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;
}
