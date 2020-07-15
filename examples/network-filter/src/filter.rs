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

use std::rc::Rc;
use std::time::Duration;

use envoy::extension::{filter::network, InstanceId, NetworkFilter, NetworkFilterStatus, Result};
use envoy::host::{log, Clock, HttpClient, HttpClientRequestHandle, HttpClientResponseOps};

use chrono::offset::Local;
use chrono::DateTime;

use super::config::SampleNetworkFilterConfig;
use super::stats::SampleNetworkFilterStats;

/// Sample network filter.
pub struct SampleNetworkFilter<'a> {
    // This example shows how multiple filter instances could share
    // the same configuration.
    config: Rc<SampleNetworkFilterConfig>,
    // This example shows how multiple filter instances could share
    // metrics.
    stats: Rc<SampleNetworkFilterStats>,
    instance_id: InstanceId,
    // This example shows how to use Time API, HTTP Client API and
    // Metrics API provided by Envoy host.
    clock: &'a dyn Clock,
    http_client: &'a dyn HttpClient,

    active_request: Option<HttpClientRequestHandle>,
    response_body_size: u64,
}

impl<'a> SampleNetworkFilter<'a> {
    /// Creates a new instance of Sample Network Filter.
    pub fn new(
        config: Rc<SampleNetworkFilterConfig>,
        stats: Rc<SampleNetworkFilterStats>,
        instance_id: InstanceId,
        clock: &'a dyn Clock,
        http_client: &'a dyn HttpClient,
    ) -> Self {
        // Inject dependencies on Envoy host APIs
        SampleNetworkFilter {
            config,
            stats,
            instance_id,
            clock,
            http_client,
            active_request: None,
            response_body_size: 0,
        }
    }
}

impl<'a> NetworkFilter for SampleNetworkFilter<'a> {
    /// Is called when a new TCP connection is opened.
    fn on_new_connection(&mut self) -> Result<NetworkFilterStatus> {
        // Update stats
        self.stats.requests_active().inc()?;

        let now: DateTime<Local> = self.clock.now()?.into();

        log::info!(
            "#{} new TCP connection starts at {} with config: {:?}",
            self.instance_id,
            now.format("%+"),
            self.config,
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
        if let Some(request) = self.active_request {
            log::info!("#{} sent outgoing request: @{}", self.instance_id, request);
        }

        Ok(NetworkFilterStatus::StopIteration)
    }

    /// Is called on response body part.
    fn on_upstream_data(
        &mut self,
        data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn network::UpstreamDataOps,
    ) -> Result<NetworkFilterStatus> {
        self.response_body_size += data_size as u64;

        Ok(NetworkFilterStatus::Continue)
    }

    /// Is called when the TCP connection is complete.
    fn on_connection_complete(&mut self) -> Result<()> {
        // Update stats
        self.stats.requests_active().dec()?;
        self.stats.requests_total().inc()?;
        self.stats
            .response_body_size_bytes()
            .record(self.response_body_size)?;

        log::info!("#{} TCP connection ended", self.instance_id);
        Ok(())
    }

    // HTTP Client API callbacks

    /// Is called when an auxiliary HTTP request sent via HTTP Client API
    /// is finally complete.
    ///
    /// Use http_client_ops to get ahold of response headers, body, etc.
    ///
    /// Use filter_ops to amend and resume TCP flow.
    fn on_http_call_response(
        &mut self,
        request: HttpClientRequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        _filter_ops: &dyn network::Ops,
        http_client_ops: &dyn HttpClientResponseOps,
    ) -> Result<()> {
        log::info!(
            "#{} received response on outgoing request: @{}",
            self.instance_id,
            request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        log::info!("     headers[count={}]:", num_headers);
        let response_headers = http_client_ops.get_http_call_response_headers()?;
        for (name, value) in &response_headers {
            log::info!("       {}: {}", name, value);
        }

        // TODO: no way to resume tcp stream
        Ok(())
    }
}
