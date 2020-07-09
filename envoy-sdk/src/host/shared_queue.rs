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

//! `Envoy` `Shared Queue API`.

use crate::abi::proxy_wasm_ext::types::{Bytes, SharedQueueHandle};
use crate::host;

pub trait Service {
    fn register_queue(&self, name: &str) -> host::Result<SharedQueueHandle>;

    fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>>;

    fn dequeue(&self, queue_id: SharedQueueHandle) -> host::Result<Option<Bytes>>;

    fn enqueue(&self, queue_id: SharedQueueHandle, value: Option<&[u8]>) -> host::Result<()>;
}

impl dyn Service {
    pub fn default() -> &'static dyn Service {
        &impls::Host
    }
}

mod impls {
    use super::Service;
    use crate::abi::proxy_wasm_ext::hostcalls;
    use crate::abi::proxy_wasm_ext::types::{Bytes, SharedQueueHandle};
    use crate::host;

    pub struct Host;

    impl Service for Host {
        fn register_queue(&self, name: &str) -> host::Result<SharedQueueHandle> {
            hostcalls::register_shared_queue(name)
        }

        fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>> {
            hostcalls::resolve_shared_queue(vm_id, name)
        }

        fn dequeue(&self, queue_id: SharedQueueHandle) -> host::Result<Option<Bytes>> {
            hostcalls::dequeue_shared_queue(queue_id)
        }

        fn enqueue(&self, queue_id: SharedQueueHandle, value: Option<&[u8]>) -> host::Result<()> {
            hostcalls::enqueue_shared_queue(queue_id, value)
        }
    }
}
