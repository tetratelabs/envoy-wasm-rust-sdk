use proxy_wasm::traits::RootContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

use access_logger::SampleAccessLogger;

// Apparently, Rust toolchain doesn't handle well exported name `_start`
// when this package is compiled to targets other than `wasm32-unknown-unknown`.
// Specifically, linking issues have been observed with targets `wasm32-wasi`
// and `x86_64-unknown-linux-gnu`, which is a blocker for unit testing.
// Therefore, export name `_start` only in the context of target `wasm32-unknown-unknown`.
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown"
    ),
    export_name = "_start"
)]
#[no_mangle]
/// Is called when a new instance of WebAssembly module is created.
extern "C" fn start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        // Inject dependencies on Envoy host APIs
        let logger = SampleAccessLogger::new(&time::ops::Host, &clients::http::ops::Host);
        Box::new(extension::access_logger::LoggerContext::new(
            logger,
            &extension::access_logger::ops::Host,
            &clients::http::ops::Host,
        ))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_start() {
        start()
    }
}
