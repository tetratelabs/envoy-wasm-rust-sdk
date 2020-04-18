use crate::host;

use proxy_wasm::types::Bytes;
use proxy_wasm::hostcalls;

pub trait Service {
    fn get_data(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)>;
    
    fn set_data(
        &self,
        key: &str,
        value: Option<&[u8]>,
        cas: Option<u32>,
    ) -> host::Result<()>;
}

struct Abi;

impl Service for Abi {
    fn get_data(&self, key: &str) -> host::Result<(Option<Bytes>, Option<u32>)> {
        hostcalls::get_shared_data(key)
    }

    fn set_data(
        &self,
        key: &str,
        value: Option<&[u8]>,
        cas: Option<u32>,
    ) -> host::Result<()> {
        hostcalls::set_shared_data(key, value, cas)
    }
}
