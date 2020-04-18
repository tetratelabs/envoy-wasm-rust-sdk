mod config;
mod filter;
mod factory;

use proxy_wasm::traits::HttpContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension;
use envoy_sdk::extension::filter::http;
use envoy_sdk::host::services::time;
use envoy_sdk::host::services::clients;

use factory::SampleHttpFilterFactory;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_http_context(|context_id, _| -> Box<dyn HttpContext> {
        let mut factory = SampleHttpFilterFactory::new(&time::ops::Host, &clients::http::ops::Host);
        let http_filter = <SampleHttpFilterFactory as extension::factory::Factory>::new_extension(&mut factory, context_id).unwrap();
        Box::new(http::FilterContext::new(http_filter, http::ops::Host))
    });
}
