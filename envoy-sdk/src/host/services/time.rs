extern crate std;
use std::time::SystemTime;

use crate::host;

pub trait Service {
    fn get_current_time(&self) -> host::Result<SystemTime>;
}

pub mod ops {
    use std::time::SystemTime;
    use crate::host;
    use proxy_wasm::hostcalls;

    pub struct Host;

    impl super::Service for Host {
        fn get_current_time(&self) -> host::Result<SystemTime> {
            hostcalls::get_current_time()
        }
    }
}
