// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

use log::info;

use envoy_sdk::extension::access_logger;
use envoy_sdk::extension::Result;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

use chrono::offset::Local;
use chrono::DateTime;

use super::config::SampleAccessLoggerConfig;

/// Sample Access Logger.
pub struct SampleAccessLogger<'a> {
    config: SampleAccessLoggerConfig,
    // This example shows how to use Time API and HTTP Client API
    // provided by Envoy host.
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,

    active_request: Option<clients::http::RequestHandle>,
}

impl<'a> SampleAccessLogger<'a> {
    /// Creates a new instance of sample access logger.
    pub fn new(
        time_service: &'a dyn time::Service,
        http_client: &'a dyn clients::http::Client,
    ) -> SampleAccessLogger<'a> {
        // Inject dependencies on Envoy host APIs
        SampleAccessLogger {
            config: SampleAccessLoggerConfig::default(),
            time_service,
            http_client,
            active_request: None,
        }
    }
}

impl<'a> access_logger::Logger for SampleAccessLogger<'a> {
    /// Is called when Envoy creates a new Listener that uses sample access logger.
    ///
    /// Use logger_ops to get ahold of configuration.
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        logger_ops: &dyn access_logger::ConfigureOps,
    ) -> Result<bool> {
        let value = match logger_ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => String::new(),
        };
        self.config = SampleAccessLoggerConfig::new(value);
        Ok(true)
    }

    /// Is called to log a complete TCP connection or HTTP request.
    ///
    /// Use logger_ops to get ahold of request/response headers,
    /// TCP connection properties, etc.
    fn on_log(&mut self, logger_ops: &dyn access_logger::LogOps) -> Result<()> {
        let current_time = self.time_service.get_current_time()?;
        let datetime: DateTime<Local> = current_time.into();

        info!(
            "logging at {} with config: {}",
            datetime.format("%+"),
            self.config.value
        );

        info!("  request headers:");
        let request_headers = logger_ops.get_request_headers()?;
        for (name, value) in &request_headers {
            info!("    {}: {}", name, value);
        }
        info!("  response headers:");
        let response_headers = logger_ops.get_response_headers()?;
        for (name, value) in &response_headers {
            info!("    {}: {}", name, value);
        }
        let upstream_address = logger_ops.get_property(vec!["upstream", "address"])?;
        let upstream_address = upstream_address
            .map(|value| String::from_utf8(value).unwrap())
            .unwrap_or_else(String::new);
        info!("  upstream info:");
        info!("    {}: {}", "upstream.address", upstream_address);

        // simulate sending a log entry off
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
            "sent request to a log collector: @{}",
            self.active_request.as_ref().unwrap()
        );

        Ok(())
    }

    // HTTP Client API callbacks

    /// Is called when an auxiliary HTTP request sent via HTTP Client API
    /// is finally complete.
    ///
    /// Use http_client_ops to get ahold of response headers, body, etc.
    fn on_http_call_response(
        &mut self,
        request: clients::http::RequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        http_client_ops: &dyn clients::http::ResponseOps,
    ) -> Result<()> {
        info!(
            "received response from a log collector on request: @{}",
            request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        info!("  headers[count={}]:", num_headers);
        let response_headers = http_client_ops.get_http_call_response_headers()?;
        for (name, value) in &response_headers {
            info!("    {}: {}", name, value);
        }

        Ok(())
    }
}
