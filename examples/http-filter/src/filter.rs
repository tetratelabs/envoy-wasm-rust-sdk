use std::rc::Rc;

use super::config::SampleHttpFilterConfig;

use log::info;

use envoy_sdk::extension::Result;
use envoy_sdk::extension::filter::http;

pub struct SampleHttpFilter {
    config: Rc<SampleHttpFilterConfig>,
    context_id: u32,
}

impl SampleHttpFilter {
    pub fn new(config: Rc<SampleHttpFilterConfig>, context_id: u32) -> SampleHttpFilter {
        SampleHttpFilter {
            config: config,
            context_id: context_id,
        }
    }
}

impl http::Filter for SampleHttpFilter {
    fn on_request_headers(&mut self, _num_headers: usize, ops: &dyn http::RequestHeadersOps) -> Result<http::FilterHeadersStatus> {
        info!("#{} -> config: {}", self.context_id, self.config.value);

        for (name, value) in &ops.get_request_headers()? {
            info!("#{} -> {}: {}", self.context_id, name, value);
        }

        match ops.get_request_header(":path")? {
            Some(path) if path == "/ping" => {
                ops.send_response(
                    200,
                    vec![("x-sample-response", "pong")],
                    Some(b"Pong!\n"),
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
        info!("#{} http exchange complete", self.context_id);
        Ok(())
    }
}
