extern crate std;
use std::prelude::v1::*;

use crate::host;

use proxy_wasm::types::Bytes;
use proxy_wasm::hostcalls;

pub trait Service {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;

    fn set_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()>;    
}

struct Abi;

impl Service for Abi {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
        hostcalls::get_property(path)
    }

    fn set_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()> {
        hostcalls::set_property(path, value)
    }
}
