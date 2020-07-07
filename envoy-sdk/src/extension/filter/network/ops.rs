// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proxy_wasm::hostcalls;
use proxy_wasm::types::{BufferType, Bytes};

use super::{DownstreamDataOps, UpstreamDataOps};
use crate::host;

pub struct Host;

impl DownstreamDataOps for Host {
    fn get_downstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::DownstreamData, start, max_size).map_err(|status| {
            host::Function::new("env", "proxy_get_buffer_bytes").call_error(status)
        })
    }
}

impl UpstreamDataOps for Host {
    fn get_upstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::UpstreamData, start, max_size).map_err(|status| {
            host::Function::new("env", "proxy_get_buffer_bytes").call_error(status)
        })
    }
}
