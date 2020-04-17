use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

use envoy_sdk::envoy::extension;
use envoy_sdk::envoy::extension::Result;
use envoy_sdk::envoy::extension::filter::http;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_http_context(|context_id, _| -> Box<dyn HttpContext> {
        Box::new(http::FilterContext::new(
            HttpHeadersFilter{context_id: context_id},
            http::ops::Host,
        ))
    });
}

struct HttpHeadersFilter {
    context_id: u32,
}

impl http::Filter for HttpHeadersFilter {
    fn on_request_headers(&mut self, _num_headers: usize, ops: &dyn http::RequestHeadersOps) -> Result<http::FilterHeadersStatus> {
        for (name, value) in &ops.get_request_headers()? {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }

        match ops.get_request_header(":path")? {
            Some(path) if path == "/hello" => {
                ops.send_response(
                    200,
                    vec![("Hello", "World"), ("Powered-By", "proxy-wasm")],
                    Some(b"Hello, World!\n"),
                )?;
                Ok(http::FilterHeadersStatus::Pause)
            }
            _ => Ok(http::FilterHeadersStatus::Continue),
        }
    }

    fn on_response_headers(&mut self, _num_headers: usize, ops: &dyn http::ResponseHeadersOps) -> Result<http::FilterHeadersStatus> {
        for (name, value) in &ops.get_response_headers()? {
            info!("#{} <- {}: {}", self.context_id, name, value);
        }
        Ok(http::FilterHeadersStatus::Continue)
    }

    fn on_exchange_complete(&mut self) -> Result<()> {
        info!("#{} completed.", self.context_id);
        Ok(())
    }
}

struct HttpHeadersFilterFactory {
}

impl extension::Factory for HttpHeadersFilterFactory {
    type Extension = HttpHeadersFilter;

    fn on_configure(&mut self, _plugin_configuration_size: usize, _ops: &dyn extension::factory::ConfigureOps) -> Result<bool> {
        Ok(true)
    }

    fn new_extension(&mut self, context_id: u32) -> Result<Box<HttpHeadersFilter>> {
        Ok(Box::new(HttpHeadersFilter{context_id: context_id}))
    }

    fn on_drain(&mut self, _ops: &dyn extension::factory::DrainOps) -> Result<bool> {
        Ok(true)
    }
}
