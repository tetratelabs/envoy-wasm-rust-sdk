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

//! `Envoy` `Time API`.

use std::time::SystemTime;

use crate::host;

/// An interface of the `Envoy` `Time Service`.
pub trait Clock {
    /// Returns current system time.
    fn now(&self) -> host::Result<SystemTime>;
}

impl dyn Clock {
    pub fn default() -> &'static dyn Clock {
        &impls::Host
    }
}

mod impls {
    use std::time::SystemTime;

    use super::Clock;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::host;

    pub(super) struct Host;

    impl Clock for Host {
        fn now(&self) -> host::Result<SystemTime> {
            hostcalls::get_current_time()
        }
    }
}
