use std::rc::Rc;
use std::time::Duration;

use super::config::SampleNetworkFilterConfig;

use log::info;

use envoy_sdk::extension::filter::network;
use envoy_sdk::extension::Result;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

extern crate chrono;
use chrono::offset::Local;
use chrono::DateTime;

pub struct SampleNetworkFilter<'a> {
    config: Rc<SampleNetworkFilterConfig>,
    instance_id: u32,
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,

    active_request: Option<clients::http::RequestHandle>,
}

impl<'a> SampleNetworkFilter<'a> {
    pub fn new(
        config: Rc<SampleNetworkFilterConfig>,
        instance_id: u32,
        time_service: &'a dyn time::Service,
        http_client: &'a dyn clients::http::Client,
    ) -> SampleNetworkFilter<'a> {
        SampleNetworkFilter {
            config,
            instance_id,
            time_service,
            http_client,
            active_request: None,
        }
    }
}

impl<'a> network::Filter for SampleNetworkFilter<'a> {
    fn on_new_connection(&mut self) -> Result<network::FilterStatus> {
        let current_time = self.time_service.get_current_time()?;
        let datetime: DateTime<Local> = current_time.into();

        info!(
            "#{} new TCP connection starts at {} with config: {}",
            self.instance_id,
            datetime.format("%+"),
            self.config.value
        );

        self.active_request = Some(self.http_client.send_request(
            "mock_service",
            vec![
                (":method", "GET"),
                (":path", "/mock"),
                (":authority", "mock.local"),
            ],
            None,
            vec![],
            Duration::from_secs(3),
        )?);
        info!(
            "#{} sent outgoing request: @{}",
            self.instance_id,
            self.active_request.as_ref().unwrap()
        );

        Ok(network::FilterStatus::Pause)
    }

    fn on_connection_complete(&mut self) -> Result<()> {
        info!("#{} TCP connection ended", self.instance_id);
        Ok(())
    }

    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        request: clients::http::RequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        _filter_ops: &dyn network::Ops,
        http_client_ops: &dyn clients::http::ResponseOps,
    ) -> Result<()> {
        info!(
            "#{} received response on outgoing request: @{}",
            self.instance_id, request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        info!("     headers[count={}]:", num_headers);
        let response_headers = http_client_ops.get_http_call_response_headers()?;
        for (name, value) in &response_headers {
            info!("       {}: {}", name, value);
        }

        // TODO: no way to resume tcp stream
        Ok(())
    }
}
