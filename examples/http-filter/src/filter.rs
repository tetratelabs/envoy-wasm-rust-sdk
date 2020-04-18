use std::rc::Rc;

use super::config::SampleHttpFilterConfig;

use log::info;

use envoy_sdk::extension::Result;
use envoy_sdk::extension::filter::http;
use envoy_sdk::host::services::time;
use envoy_sdk::host::services::clients;

extern crate chrono;
use chrono::offset::Local;
use chrono::DateTime;

pub struct SampleHttpFilter<'a> {
    config: Rc<SampleHttpFilterConfig>,
    instance_id: u32,
    time_service: &'a dyn time::Service,
    _http_client: &'a dyn clients::http::Client,
}

impl<'a> SampleHttpFilter<'a> {
    pub fn new(config: Rc<SampleHttpFilterConfig>, instance_id: u32, time_service: &'a dyn time::Service, http_client: &'a dyn clients::http::Client) -> SampleHttpFilter<'a> {
        SampleHttpFilter {
            config: config,
            instance_id: instance_id,
            time_service: time_service,
            _http_client: http_client,
        }
    }
}

impl<'a> http::Filter for SampleHttpFilter<'a> {
    fn on_request_headers(&mut self, _num_headers: usize, ops: &dyn http::RequestHeadersOps) -> Result<http::FilterHeadersStatus> {
        let current_time = self.time_service.get_current_time()?;
        let datetime: DateTime<Local> = current_time.into();

        info!("#{} new http exchange starts at {} with config: {}", self.instance_id, datetime.format("%+"), self.config.value);

        for (name, value) in &ops.get_request_headers()? {
            info!("#{} -> {}: {}", self.instance_id, name, value);
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
            info!("#{} <- {}: {}", self.instance_id, name, value);
        }
        Ok(http::FilterHeadersStatus::Continue)
    }

    fn on_exchange_complete(&mut self) -> Result<()> {
        info!("#{} http exchange complete", self.instance_id);
        Ok(())
    }
}
