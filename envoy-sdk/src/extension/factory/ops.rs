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

extern crate std;

use std::prelude::v1::*;

use proxy_wasm::hostcalls;
use proxy_wasm::types::Bytes;

use crate::host;

pub struct Host;

impl super::ConfigureOps for Host {
    fn get_configuration(&self) -> host::Result<Option<Bytes>> {
        hostcalls::get_configuration().map_err(|status| ("proxy_get_configuration", status))
    }
}

impl super::DrainOps for Host {
    fn done(&self) -> host::Result<()> {
        hostcalls::done().map_err(|status| ("proxy_done", status))
    }
}
