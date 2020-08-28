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

use crate::host::{self, ByteString};

pub use crate::abi::proxy_wasm::types::SharedQueueHandle;

/// An interface of the `Envoy` `Shared Queue API`.
///
/// Basic usage of [`SharedQueue`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::SharedQueue;
///
/// let shared_queue = SharedQueue::default();
///
/// let queue_handle = shared_queue.register("shared_queue")?;
///
/// shared_queue.enqueue(queue_handle, b"some value")?;
/// # Ok(())
/// # }
/// ```
///
/// Injecting [`SharedQueue`] into a HTTP Filter as a dependency:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::host::SharedQueue;
///
/// struct MyHttpFilter<'a> {
///     shared_queue: &'a dyn SharedQueue,
/// }
///
/// impl<'a> MyHttpFilter<'a> {
///     /// Creates a new instance parameterized with a given [`SharedQueue`] implementation.
///     pub fn new(shared_queue: &'a dyn SharedQueue) -> Self {
///         MyHttpFilter { shared_queue }
///     }
///
///     /// Creates a new instance parameterized with the default [`SharedQueue`] implementation.
///     pub fn default() -> Self {
///         Self::new(SharedQueue::default())
///     }
/// }
/// ```
///
/// [`SharedQueue`]: trait.SharedQueue.html
pub trait SharedQueue {
    fn register(&self, name: &str) -> host::Result<SharedQueueHandle>;

    fn lookup(&self, vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>>;

    fn dequeue(&self, queue_id: SharedQueueHandle) -> host::Result<Option<ByteString>>;

    fn enqueue(&self, queue_id: SharedQueueHandle, value: &[u8]) -> host::Result<()>;
}

impl dyn SharedQueue {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn SharedQueue {
        &impls::Host
    }
}

mod impls {
    use super::SharedQueue;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::SharedQueueHandle;
    use crate::host::{self, ByteString};

    pub(super) struct Host;

    impl SharedQueue for Host {
        fn register(&self, name: &str) -> host::Result<SharedQueueHandle> {
            hostcalls::register_shared_queue(name)
        }

        fn lookup(&self, vm_id: &str, name: &str) -> host::Result<Option<SharedQueueHandle>> {
            hostcalls::resolve_shared_queue(vm_id, name)
        }

        fn dequeue(&self, queue_id: SharedQueueHandle) -> host::Result<Option<ByteString>> {
            hostcalls::dequeue_shared_queue(queue_id)
        }

        fn enqueue(&self, queue_id: SharedQueueHandle, value: &[u8]) -> host::Result<()> {
            hostcalls::enqueue_shared_queue(queue_id, value)
        }
    }
}
