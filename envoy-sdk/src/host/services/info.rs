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

use crate::host;

use proxy_wasm::types::Bytes;

pub trait Service {
    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;

    fn set_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()>;
}

pub mod ops {
    use crate::host;
    use proxy_wasm::hostcalls;
    use proxy_wasm::types::Bytes;

    pub struct Host;

    impl super::Service for Host {
        fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
            hostcalls::get_property(path).map_err(|status| ("proxy_get_property", status))
        }

        fn set_property(&self, path: Vec<&str>, value: Option<&[u8]>) -> host::Result<()> {
            hostcalls::set_property(path, value).map_err(|status| ("proxy_set_property", status))
        }
    }
}
