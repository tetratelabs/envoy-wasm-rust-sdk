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

use std::time::SystemTime;

use crate::host;

pub trait Service {
    fn get_current_time(&self) -> host::Result<SystemTime>;
}

pub mod ops {
    use std::time::SystemTime;

    use proxy_wasm::hostcalls;

    use super::Service;
    use crate::host;

    pub struct Host;

    impl Service for Host {
        fn get_current_time(&self) -> host::Result<SystemTime> {
            hostcalls::get_current_time()
                .map_err(|status| ("proxy_get_current_time_nanoseconds", status))
        }
    }
}
