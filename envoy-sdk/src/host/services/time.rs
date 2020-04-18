extern crate std;
use std::time::SystemTime;

use crate::host;
use proxy_wasm::hostcalls;

pub trait Service {
    fn get_current_time(&self) -> host::Result<SystemTime>;
}

struct Abi;

impl Service for Abi {
    fn get_current_time(&self) -> host::Result<SystemTime> {
        hostcalls::get_current_time()
    }
}
