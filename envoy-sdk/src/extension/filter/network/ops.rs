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

use crate::abi::proxy_wasm::hostcalls;
use crate::abi::proxy_wasm::types::BufferType;

use super::{
    ConnectionCompleteOps, DownstreamCloseOps, DownstreamDataOps, UpstreamCloseOps, UpstreamDataOps, BufferAction,
};
use crate::host::{self, Bytes};

pub(super) struct Host;

impl DownstreamDataOps for Host {
    fn downstream_data(&self, start: usize, max_size: usize) -> host::Result<Bytes> {
        hostcalls::get_buffer(BufferType::DownstreamData, start, max_size)
    }

    fn mutate_downstream_data(&self, action: BufferAction) -> host::Result<()> {
        action.execute(|start: usize, max_size: usize, data: &[u8]| {
            hostcalls::set_buffer(BufferType::DownstreamData, start, max_size, data)
        })
    }
}

impl UpstreamDataOps for Host {
    fn upstream_data(&self, start: usize, max_size: usize) -> host::Result<Bytes> {
        hostcalls::get_buffer(BufferType::UpstreamData, start, max_size)
    }

    fn mutate_upstream_data(&self, action: BufferAction) -> host::Result<()> {
        action.execute(|start: usize, max_size: usize, data: &[u8]| {
            hostcalls::set_buffer(BufferType::UpstreamData, start, max_size, data)
        })
    }
}

impl DownstreamCloseOps for Host {}

impl UpstreamCloseOps for Host {}

impl ConnectionCompleteOps for Host {}
