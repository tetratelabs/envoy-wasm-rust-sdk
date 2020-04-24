#![cfg(not(test))]

use proxy_wasm::traits::StreamContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension;
use envoy_sdk::extension::filter::network;
use envoy_sdk::host::services::time;
use envoy_sdk::host::services::clients;

use crate::factory::SampleNetworkFilterFactory;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_stream_context(|context_id, _| -> Box<dyn StreamContext> {
        // TODO: at the moment, extension configuration is ignored since it belongs to the RootContext 
        // but `proxy-wasm` doesn't provide any way to associate StreamContext with its parent RootContext
        let mut factory = SampleNetworkFilterFactory::new(&time::ops::Host, &clients::http::ops::Host);
        let network_filter = <SampleNetworkFilterFactory as extension::factory::Factory>::new_extension(&mut factory, context_id).unwrap();
        Box::new(network::FilterContext::new(network_filter, &network::ops::Host, &clients::http::ops::Host))
    });
}
