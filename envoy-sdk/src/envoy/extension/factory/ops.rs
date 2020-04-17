extern crate std;
use std::prelude::v1::*;

use crate::envoy::host;

use proxy_wasm::types::Bytes;
use proxy_wasm::hostcalls;

pub struct Host;

impl super::ConfigureOps for Host {
    fn get_configuration(&self) -> host::Result<Option<Bytes>> {
        hostcalls::get_configuration()
    }
}

impl super::DrainOps for Host {
    fn done(&self) -> host::Result<()> {
        hostcalls::done()
    }
}
