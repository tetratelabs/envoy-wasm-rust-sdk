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

use envoy::extension::filter::http;
use envoy::extension::{HttpFilter, InstanceId, Result};
use envoy::host::{
    log, Clock, HttpClient, HttpClientRequestHandle, HttpClientResponseOps, StreamInfo,
};

use chrono::offset::Local;
use chrono::DateTime;

use super::config::SampleHttpFilterConfig;
use super::stats::SampleHttpFilterStats;

// Sample HTTP Filter.
pub struct SampleHttpFilter<'a> {
    // This example shows how multiple filter instances could share
    // the same configuration.
    config: Rc<SampleHttpFilterConfig>,
    // This example shows how multiple filter instances could share
    // metrics.
    stats: Rc<SampleHttpFilterStats>,
    instance_id: InstanceId,
    // This example shows how to use Time API, HTTP Client API and
    // Metrics API provided by Envoy host.
    clock: &'a dyn Clock,
    http_client: &'a dyn HttpClient,
    stream_info: &'a dyn StreamInfo,

    active_request: Option<HttpClientRequestHandle>,
    response_body_size: u64,
}

impl<'a> SampleHttpFilter<'a> {
    /// Creates a new instance of Sample HTTP Filter.
    pub fn new(
        config: Rc<SampleHttpFilterConfig>,
        stats: Rc<SampleHttpFilterStats>,
        instance_id: InstanceId,
        clock: &'a dyn Clock,
        http_client: &'a dyn HttpClient,
        stream_info: &'a dyn StreamInfo,
    ) -> Self {
        // Inject dependencies on Envoy host APIs
        SampleHttpFilter {
            config,
            stats,
            instance_id,
            clock,
            http_client,
            stream_info,
            active_request: None,
            response_body_size: 0,
        }
    }
}

impl<'a> HttpFilter for SampleHttpFilter<'a> {
    /// Is called when HTTP request headers have been received.
    ///
    /// Use filter_ops to access and mutate request headers.
    fn on_request_headers(
        &mut self,
        _num_headers: usize,
        filter_ops: &dyn http::RequestHeadersOps,
    ) -> Result<http::FilterHeadersStatus> {
        // Update stats
        self.stats.requests_active().inc()?;

        let now: DateTime<Local> = self.clock.now()?.into();

        log::info!(
            "#{} new http exchange starts at {} with config: {:?}",
            self.instance_id,
            now.format("%+"),
            self.config,
        );

        log::info!("#{} observing request headers", self.instance_id);
        for (name, value) in &filter_ops.request_headers()? {
            log::info!("#{} -> {}: {}", self.instance_id, name, value);
        }

        log::info!("  connection.id: {:?}", self.stream_info.connection().id()?);
        log::info!("  request.id: {:?}", self.stream_info.request().id()?);
        log::info!(
            "  listener.traffic_direction: {:?}",
            self.stream_info.listener().traffic_direction()?
        );
        log::info!("  route.name: {:?}", self.stream_info.route().name()?);
        log::info!("  cluster.name: {:?}", self.stream_info.cluster().name()?);
        log::info!("  plugin.name: {:?}", self.stream_info.plugin().name()?);

        match filter_ops.request_header(":path")? {
            Some(path) if path == "/ping" => {
                filter_ops.send_response(
                    200,
                    &[("x-sample-response", "pong")],
                    Some(b"Pong!\n"),
                )?;
                Ok(http::FilterHeadersStatus::StopIteration)
            }
            Some(path) if path == "/secret" => {
                self.active_request = Some(self.http_client.send_request(
                    "mock_service",
                    &[
                        (":method", "GET"),
                        (":path", "/authz"),
                        (":authority", "mock.local"),
                    ],
                    None,
                    None,
                    Duration::from_secs(3),
                )?);
                if let Some(request) = self.active_request {
                    log::info!(
                        "#{} sent authorization request: @{}",
                        self.instance_id,
                        request,
                    );
                }
                log::info!("#{} suspending http exchange processing", self.instance_id);
                Ok(http::FilterHeadersStatus::StopIteration)
            }
            _ => Ok(http::FilterHeadersStatus::Continue),
        }
    }

    /// Is called when HTTP response headers have been received.
    ///
    /// Use filter_ops to access and mutate response headers.
    fn on_response_headers(
        &mut self,
        _num_headers: usize,
        filter_ops: &dyn http::ResponseHeadersOps,
    ) -> Result<http::FilterHeadersStatus> {
        log::info!("#{} observing response headers", self.instance_id);
        for (name, value) in &filter_ops.response_headers()? {
            log::info!("#{} <- {}: {}", self.instance_id, name, value);
        }
        Ok(http::FilterHeadersStatus::Continue)
    }

    /// Is called on response body part.
    fn on_response_body(
        &mut self,
        data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn http::ResponseBodyOps,
    ) -> Result<http::FilterDataStatus> {
        self.response_body_size += data_size as u64;

        Ok(http::FilterDataStatus::Continue)
    }

    /// Is called when HTTP exchange is complete.
    fn on_exchange_complete(&mut self, _ops: &dyn http::ExchangeCompleteOps) -> Result<()> {
        // Update stats
        self.stats.requests_active().dec()?;
        self.stats.requests_total().inc()?;
        self.stats
            .response_body_size_bytes()
            .record(self.response_body_size)?;

        log::info!("#{} http exchange complete", self.instance_id);
        log::info!(
            "  response.flags: {:?}",
            self.stream_info.response().flags()?
        );
        Ok(())
    }

    // HTTP Client API callbacks

    /// Is called when an auxiliary HTTP request sent via HTTP Client API
    /// is finally complete.
    ///
    /// Use http_client_ops to get ahold of response headers, body, etc.
    ///
    /// Use filter_ops to amend and resume HTTP exchange.
    fn on_http_call_response(
        &mut self,
        request: HttpClientRequestHandle,
        num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        filter_ops: &dyn http::Ops,
        http_client_ops: &dyn HttpClientResponseOps,
    ) -> Result<()> {
        log::info!(
            "#{} received response on authorization request: @{}",
            self.instance_id,
            request
        );
        assert!(self.active_request == Some(request));
        self.active_request = None;

        log::info!("     headers[count={}]:", num_headers);
        let response_headers = http_client_ops.http_call_response_headers()?;
        for (name, value) in &response_headers {
            log::info!("       {}: {}", name, value);
        }

        log::info!("#{} resuming http exchange processing", self.instance_id);
        filter_ops.resume_request()?;
        Ok(())
    }
}
