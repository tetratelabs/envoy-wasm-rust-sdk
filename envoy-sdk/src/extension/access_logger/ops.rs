extern crate std;
use std::prelude::v1::*;

use crate::host;

use proxy_wasm::types::Bytes;
use proxy_wasm::hostcalls;

pub struct Host;

impl super::LogOps for Host {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
        hostcalls::get_property(path).map_err(|status| ("proxy_get_property", status))
    }
}
