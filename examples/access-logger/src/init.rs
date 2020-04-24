#![cfg(not(test))]

use proxy_wasm::traits::RootContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension::access_logger;
use envoy_sdk::host::services::time;
use envoy_sdk::host::services::clients;

use crate::logger::SampleAccessLogger;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        let logger = SampleAccessLogger::new(&time::ops::Host, &clients::http::ops::Host);
        Box::new(access_logger::LoggerContext::new(logger, &access_logger::ops::Host, &clients::http::ops::Host))
    });
}
