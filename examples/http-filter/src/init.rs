use proxy_wasm::traits::HttpContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension;
use envoy_sdk::extension::filter::http;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

use crate::factory::SampleHttpFilterFactory;

/// Is called when a new instance of WebAssembly module is created.
#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_http_context(|context_id, _| -> Box<dyn HttpContext> {
        // TODO: at the moment, extension configuration is ignored since it belongs to the RootContext
        // but `proxy-wasm` doesn't provide any way to associate HttpContext with its parent RootContext

        // Inject dependencies on Envoy host APIs
        let mut factory = SampleHttpFilterFactory::new(&time::ops::Host, &clients::http::ops::Host);
        let http_filter = <SampleHttpFilterFactory as extension::factory::Factory>::new_extension(
            &mut factory,
            extension::InstanceId::from(context_id),
        )
        .unwrap();
        Box::new(http::FilterContext::new(
            http_filter,
            &http::ops::Host,
            &clients::http::ops::Host,
        ))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_start() {
        _start()
    }
}
