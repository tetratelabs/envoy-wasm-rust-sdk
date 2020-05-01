pub mod services;

/// The type returned by host methods.
pub type Result<T> = core::result::Result<T, (&'static str, proxy_wasm::types::Status)>;
