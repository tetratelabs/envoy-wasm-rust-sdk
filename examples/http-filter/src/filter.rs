use std::rc::Rc;
use std::time::Duration;

use super::config::SampleHttpFilterConfig;

use log::info;

use envoy_sdk::extension::filter::http;
use envoy_sdk::extension::Result;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

extern crate chrono;
use chrono::offset::Local;
use chrono::DateTime;

pub struct SampleHttpFilter<'a> {
    config: Rc<SampleHttpFilterConfig>,
    instance_id: u32,
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,

    active_request: Option<clients::http::RequestHandle>,
}

impl<'a> SampleHttpFilter<'a> {
    pub fn new(
        config: Rc<SampleHttpFilterConfig>,
        instance_id: u32,
        time_service: &'a dyn time::Service,
        http_client: &'a dyn clients::http::Client,
    ) -> SampleHttpFilter<'a> {
        SampleHttpFilter {
            config: config,
            instance_id: instance_id,
            time_service: time_service,
            http_client: http_client,
            active_request: None,
        }
    }
}

impl<'a> http::Filter for SampleHttpFilter<'a> {
    fn on_request_headers(
        &mut self,
        _num_headers: usize,
        filter_ops: &dyn http::RequestHeadersOps,
    ) -> Result<http::FilterHeadersStatus> {
        let current_time = self.time_service.get_current_time()?;
        let datetime: DateTime<Local> = current_time.into();

        info!(
            "#{} new http exchange starts at {} with config: {}",
            self.instance_id,
            datetime.format("%+"),
            self.config.value
        );

        info!("#{} observing request headers", self.instance_id);
        for (name, value) in &filter_ops.get_request_headers()? {
            info!("#{} -> {}: {}", self.instance_id, name, value);
        }

        match filter_ops.get_request_header(":path")? {
            Some(path) if path == "/ping" => {
                filter_ops.send_response(
                    200,
                    vec![("x-sample-response", "pong")],
                    Some(b"Pong!\n"),
                )?;
                Ok(http::FilterHeadersStatus::Pause)
            }
            Some(path) if path == "/secret" => {
                self.active_request = Some(self.http_client.send_request(
                    "mock_service",
                    vec![
                        (":method", "GET"),
                        (":path", "/authz"),
                        (":authority", "mock.local"),
                    ],
                    None,
                    vec![],
                    Duration::from_secs(3),
                )?);
                info!(
                    "#{} sent authorization request: @{}",
                    self.instance_id,
                    self.active_request.as_ref().unwrap()
                );
                info!("#{} suspending http exchange processing", self.instance_id);
                Ok(http::FilterHeadersStatus::Pause)
            }
            _ => Ok(http::FilterHeadersStatus::Continue),
        }
    }

    fn on_response_headers(
        &mut self,
        _num_headers: usize,
        filter_ops: &dyn http::ResponseHeadersOps,
    ) -> Result<http::FilterHeadersStatus> {
        info!("#{} observing response headers", self.instance_id);
        for (name, value) in &filter_ops.get_response_headers()? {
            info!("#{} <- {}: {}", self.instance_id, name, value);
        }
        Ok(http::FilterHeadersStatus::Continue)
    }

    fn on_exchange_complete(&mut self) -> Result<()> {
        info!("#{} http exchange complete", self.instance_id);
        Ok(())
    }

    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        request: clients::http::RequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        filter_ops: &dyn http::Ops,
        http_client_ops: &dyn clients::http::ResponseOps,
    ) -> Result<()> {
        info!(
            "#{} received response on authorization request: @{}",
            self.instance_id, request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        info!("     headers[count={}]:", num_headers);
        let response_headers = http_client_ops.get_http_call_response_headers()?;
        for (name, value) in &response_headers {
            info!("       {}: {}", name, value);
        }

        info!("#{} resuming http exchange processing", self.instance_id);
        filter_ops.resume_request()?;
        Ok(())
    }
}
