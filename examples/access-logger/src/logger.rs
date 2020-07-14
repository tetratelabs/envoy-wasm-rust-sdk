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

use std::convert::TryFrom;
use std::time::Duration;

use envoy::host::log::info;

use envoy::extension::access_logger;
use envoy::extension::{ConfigStatus, Result};
use envoy::host::{http::client as http_client, stats, Clock};

use chrono::offset::Local;
use chrono::DateTime;

use super::config::SampleAccessLoggerConfig;
use super::stats::SampleAccessLoggerStats;

/// Sample Access Logger.
pub struct SampleAccessLogger<'a> {
    config: SampleAccessLoggerConfig,
    stats: SampleAccessLoggerStats,
    // This example shows how to use Time API, HTTP Client API and
    // Metrics API provided by Envoy host.
    clock: &'a dyn Clock,
    http_client: &'a dyn http_client::Client,

    active_request: Option<http_client::RequestHandle>,
}

impl<'a> SampleAccessLogger<'a> {
    /// Creates a new instance of Sample Access Logger.
    pub fn new(
        clock: &'a dyn Clock,
        http_client: &'a dyn http_client::Client,
        metrics_service: &'a dyn stats::Service,
    ) -> Result<Self> {
        let stats = SampleAccessLoggerStats::new(
            metrics_service.counter("examples.access_logger.requests_total")?,
            metrics_service.gauge("examples.access_logger.reports_active")?,
            metrics_service.counter("examples.access_logger.reports_total")?,
        );
        // Inject dependencies on Envoy host APIs
        Ok(SampleAccessLogger {
            config: SampleAccessLoggerConfig::default(),
            stats,
            clock,
            http_client,
            active_request: None,
        })
    }

    /// Creates a new instance of Sample Access Logger
    /// bound to the actual Envoy ABI.
    pub fn default() -> Result<Self> {
        Self::new(
            Clock::default(),
            http_client::Client::default(),
            stats::Service::default(),
        )
    }
}

impl<'a> access_logger::Logger for SampleAccessLogger<'a> {
    /// The reference name for Sample Access Logger.
    ///
    /// This name appears in `Envoy` configuration as a value of `root_id` field
    /// (also known as `group_name`).
    const NAME: &'static str = "examples.access_logger";

    /// Is called when Envoy creates a new Listener that uses Sample Access Logger.
    ///
    /// Use logger_ops to get ahold of configuration.
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        logger_ops: &dyn access_logger::ConfigureOps,
    ) -> Result<ConfigStatus> {
        self.config = match logger_ops.get_configuration()? {
            Some(bytes) => SampleAccessLoggerConfig::try_from(bytes.as_slice())?,
            None => SampleAccessLoggerConfig::default(),
        };
        Ok(ConfigStatus::Accepted)
    }

    /// Is called to log a complete TCP connection or HTTP request.
    ///
    /// Use logger_ops to get ahold of request/response headers,
    /// TCP connection properties, etc.
    fn on_log(&mut self, logger_ops: &dyn access_logger::LogOps) -> Result<()> {
        // Update stats
        self.stats.requests_total().inc()?;

        let now: DateTime<Local> = self.clock.now()?.into();

        info!(
            "logging at {} with config: {:?}",
            now.format("%+"),
            self.config,
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
            .map(String::from_utf8)
            .transpose()?
            .unwrap_or_else(String::default);
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
        if let Some(request) = self.active_request {
            info!("sent request to a log collector: @{}", request,);
        }
        // Update stats
        self.stats.reports_active().inc()?;

        Ok(())
    }

    // HTTP Client API callbacks

    /// Is called when an auxiliary HTTP request sent via HTTP Client API
    /// is finally complete.
    ///
    /// Use http_client_ops to get ahold of response headers, body, etc.
    fn on_http_call_response(
        &mut self,
        request: http_client::RequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        http_client_ops: &dyn http_client::ResponseOps,
    ) -> Result<()> {
        info!(
            "received response from a log collector on request: @{}",
            request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        // Update stats
        self.stats.reports_active().dec()?;
        self.stats.reports_total().inc()?;

        info!("  headers[count={}]:", num_headers);
        let response_headers = http_client_ops.get_http_call_response_headers()?;
        for (name, value) in &response_headers {
            info!("    {}: {}", name, value);
        }

        Ok(())
    }
}
