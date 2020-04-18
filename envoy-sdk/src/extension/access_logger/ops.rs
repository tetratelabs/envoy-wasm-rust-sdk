extern crate std;
use std::prelude::v1::*;

use crate::host;
use host::services::info;

use proxy_wasm::types::Bytes;

impl super::LogOps for info::ops::Host {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
        <_ as info::Service>::get_property(self, path)
    }
}
