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

use proxy_wasm::types::Bytes;

use crate::host;

pub trait Service {
    fn register_queue(&self, name: &str) -> host::Result<u32>;

    fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<u32>>;

    fn dequeue(&self, queue_id: u32) -> host::Result<Option<Bytes>>;

    fn enqueue(&self, queue_id: u32, value: Option<&[u8]>) -> host::Result<()>;
}

pub mod ops {
    use crate::host;
    use proxy_wasm::hostcalls;
    use proxy_wasm::types::Bytes;

    pub struct Host;

    impl super::Service for Host {
        fn register_queue(&self, name: &str) -> host::Result<u32> {
            hostcalls::register_shared_queue(name)
                .map_err(|status| ("proxy_register_shared_queue", status))
        }

        fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<u32>> {
            hostcalls::resolve_shared_queue(vm_id, name)
                .map_err(|status| ("proxy_resolve_shared_queue", status))
        }

        fn dequeue(&self, queue_id: u32) -> host::Result<Option<Bytes>> {
            hostcalls::dequeue_shared_queue(queue_id)
                .map_err(|status| ("proxy_dequeue_shared_queue", status))
        }

        fn enqueue(&self, queue_id: u32, value: Option<&[u8]>) -> host::Result<()> {
            hostcalls::enqueue_shared_queue(queue_id, value)
                .map_err(|status| ("proxy_enqueue_shared_queue", status))
        }
    }

    // TODO: fn on_queue_ready(&mut self, _queue_id: u32) {}
}
