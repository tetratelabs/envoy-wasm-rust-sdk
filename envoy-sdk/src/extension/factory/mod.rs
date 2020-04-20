extern crate std;
use std::prelude::v1::*;

pub mod ops;
pub mod context;
pub use context::FactoryContext; 

use crate::host;
use crate::extension::Result;

use proxy_wasm::types::Bytes;

pub trait Factory {
    type Extension;

    fn on_configure(&mut self, _configuration_size: usize, _ops: &dyn ConfigureOps) -> Result<bool> {
        Ok(true)
    }

    fn new_extension(&mut self, _instance_id: u32) -> Result<Self::Extension>;

    fn on_drain(&mut self, _ops: &dyn DrainOps) -> Result<bool> {
        Ok(true)
    }
}

pub trait ConfigureOps {
    fn get_configuration(&self) -> host::Result<Option<Bytes>>;
}

pub trait DrainOps {
    fn done(&self) -> host::Result<()>;
}

pub trait Ops: ConfigureOps + DrainOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps;

    fn as_done_ops(&self) -> &dyn DrainOps;
}

impl<T> Ops for T where T: ConfigureOps + DrainOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps { self }

    fn as_done_ops(&self) -> &dyn DrainOps { self }
}
