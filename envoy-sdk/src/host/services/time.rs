extern crate std;
use std::time::SystemTime;

use crate::host;

pub trait Service {
    fn get_current_time(&self) -> host::Result<SystemTime>;
}

pub mod ops {
    use crate::host;
    use proxy_wasm::hostcalls;
    use std::time::SystemTime;

    pub struct Host;

    impl super::Service for Host {
        fn get_current_time(&self) -> host::Result<SystemTime> {
            hostcalls::get_current_time()
                .map_err(|status| ("proxy_get_current_time_nanoseconds", status))
        }
    }
}
