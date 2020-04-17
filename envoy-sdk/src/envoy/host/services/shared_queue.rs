use crate::envoy::host;

use proxy_wasm::types::Bytes;
use proxy_wasm::hostcalls;

pub trait Service {
    fn register_queue(&self, name: &str) -> host::Result<u32>;

    fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<u32>>;

    fn dequeue(&self, queue_id: u32) -> host::Result<Option<Bytes>>;

    fn enqueue(&self, queue_id: u32, value: Option<&[u8]>) -> host::Result<()>;
}

struct Abi;

impl Service for Abi {
    fn register_queue(&self, name: &str) -> host::Result<u32> {
        hostcalls::register_shared_queue(name)
    }

    fn lookup_queue(&self, vm_id: &str, name: &str) -> host::Result<Option<u32>> {
        hostcalls::resolve_shared_queue(vm_id, name)
    }

    fn dequeue(&self, queue_id: u32) -> host::Result<Option<Bytes>> {
        hostcalls::dequeue_shared_queue(queue_id)
    }

    fn enqueue(&self, queue_id: u32, value: Option<&[u8]>) -> host::Result<()> {
        hostcalls::enqueue_shared_queue(queue_id, value)
    }
}

// TODO: fn on_queue_ready(&mut self, _queue_id: u32) {}
