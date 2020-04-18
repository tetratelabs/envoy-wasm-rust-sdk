use crate::host;

use proxy_wasm::types::Bytes;

pub trait Service {
    fn get_data(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)>;
    
    fn set_data(
        &self,
        key: &str,
        value: Option<&[u8]>,
        cas: Option<u32>,
    ) -> host::Result<()>;
}

pub mod ops {
    use crate::host;
    use proxy_wasm::hostcalls;
    use proxy_wasm::types::Bytes;

    pub struct Host;

    impl super::Service for Host {
        fn get_data(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)> {
            hostcalls::get_shared_data(key).map_err(|status| ("proxy_get_shared_data", status))
        }

        fn set_data(
            &self,
            key: &str,
            value: Option<&[u8]>,
            cas: Option<u32>,
        ) -> host::Result<()> {
            hostcalls::set_shared_data(key, value, cas).map_err(|status| ("proxy_set_shared_data", status))
        }
    }
}
