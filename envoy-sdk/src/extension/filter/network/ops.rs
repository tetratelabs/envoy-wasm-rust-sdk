use crate::host;

use proxy_wasm::types::{BufferType, Bytes};
use proxy_wasm::hostcalls;

pub struct Host;

impl super::DownstreamDataOps for Host {
    fn get_downstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::DownstreamData, start, max_size).map_err(|status| ("proxy_get_buffer_bytes", status))
    }
}

impl super::UpstreamDataOps for Host {
    fn get_upstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::UpstreamData, start, max_size).map_err(|status| ("proxy_get_buffer_bytes", status))
    }
}
