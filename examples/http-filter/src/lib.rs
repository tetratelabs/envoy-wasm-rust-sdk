use std::rc::Rc;

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
        let mut factory = HttpHeadersFilterFactory::new();
        let extension = <HttpHeadersFilterFactory as extension::factory::Factory>::new_extension(&mut factory, context_id).unwrap();
        Box::new(http::FilterContext::new(extension, http::ops::Host))
    });
}

struct HttpHeadersFilterConfig {
    pub value: String,
}

impl HttpHeadersFilterConfig {
    fn new(value: String) -> HttpHeadersFilterConfig {
        HttpHeadersFilterConfig{value: value}
    }
}

impl Default for HttpHeadersFilterConfig {
    fn default() -> Self {
        HttpHeadersFilterConfig{value: "".to_string()}
    }
}

struct HttpHeadersFilter {
    config: Rc<HttpHeadersFilterConfig>,
    context_id: u32,
}

impl HttpHeadersFilter {
    fn new(config: Rc<HttpHeadersFilterConfig>, context_id: u32) -> HttpHeadersFilter {
        HttpHeadersFilter {
            config: config,
            context_id: context_id,
        }
    }
}

impl http::Filter for HttpHeadersFilter {
    fn on_request_headers(&mut self, _num_headers: usize, ops: &dyn http::RequestHeadersOps) -> Result<http::FilterHeadersStatus> {
        info!("#{} -> config: {}", self.context_id, self.config.value);

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
    config: Rc<HttpHeadersFilterConfig>,
}

impl HttpHeadersFilterFactory {
    fn new() -> HttpHeadersFilterFactory {
        HttpHeadersFilterFactory{
            config: Rc::new(HttpHeadersFilterConfig::default())
        }
    }
}

impl extension::Factory for HttpHeadersFilterFactory {
    type Extension = HttpHeadersFilter;

    fn on_configure(&mut self, _configuration_size: usize, ops: &dyn extension::factory::ConfigureOps) -> Result<bool> {
        let config = match ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => "".to_string(),
        };
        self.config = Rc::new(HttpHeadersFilterConfig::new(config));
        Ok(true)
    }

    fn new_extension(&mut self, context_id: u32) -> Result<HttpHeadersFilter> {
        Ok(HttpHeadersFilter::new(Rc::clone(&self.config), context_id))
    }
}
