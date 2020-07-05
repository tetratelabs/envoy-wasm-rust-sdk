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

use std::fmt;

use proxy_wasm::types::Bytes;

use crate::host;

/// Opaque identifier of a Queue.
#[derive(PartialEq, Eq)]
pub struct QueueHandle(u32);

impl From<u32> for QueueHandle {
    fn from(token_id: u32) -> Self {
        QueueHandle(token_id)
    }
}

impl fmt::Display for QueueHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Service {
    fn register_queue(&self, name: &str) -> host::Result<QueueHandle>;

    fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<QueueHandle>>;

    fn dequeue(&self, queue_id: QueueHandle) -> host::Result<Option<Bytes>>;

    fn enqueue(&self, queue_id: QueueHandle, value: Option<&[u8]>) -> host::Result<()>;
}

pub mod ops {
    use proxy_wasm::hostcalls;
    use proxy_wasm::types::Bytes;

    use super::{QueueHandle, Service};
    use crate::host;

    pub struct Host;

    impl Service for Host {
        fn register_queue(&self, name: &str) -> host::Result<QueueHandle> {
            hostcalls::register_shared_queue(name)
                .map_err(|status| ("proxy_register_shared_queue", status))
                .map(QueueHandle::from)
        }

        fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<QueueHandle>> {
            hostcalls::resolve_shared_queue(vm_id, name)
                .map_err(|status| ("proxy_resolve_shared_queue", status))
                .map(|o| o.map(QueueHandle::from))
        }

        fn dequeue(&self, queue_id: QueueHandle) -> host::Result<Option<Bytes>> {
            hostcalls::dequeue_shared_queue(queue_id.0)
                .map_err(|status| ("proxy_dequeue_shared_queue", status))
        }

        fn enqueue(&self, queue_id: QueueHandle, value: Option<&[u8]>) -> host::Result<()> {
            hostcalls::enqueue_shared_queue(queue_id.0, value)
                .map_err(|status| ("proxy_enqueue_shared_queue", status))
        }
    }

    // TODO: fn on_queue_ready(&mut self, _queue_id: u32) {}
}
