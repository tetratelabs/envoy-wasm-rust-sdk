use crate::extension::Result;
use crate::host;

use proxy_wasm::hostcalls;
use proxy_wasm::types::Bytes;

pub trait Lifecycle {
    fn on_start(&mut self, _vm_configuration_size: usize, _ops: &dyn StartOps) -> Result<bool> {
        Ok(true)
    }
}

pub trait StartOps {
    fn get_configuration(&self) -> host::Result<Option<Bytes>>;
}

pub struct Host;

impl StartOps for Host {
    fn get_configuration(&self) -> host::Result<Option<Bytes>> {
        hostcalls::get_configuration().map_err(|status| ("proxy_get_configuration", status))
    }
}
