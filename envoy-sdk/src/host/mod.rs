pub mod services;

pub type Result<T> = core::result::Result<T, (&'static str, proxy_wasm::types::Status)>;
