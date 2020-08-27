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
use std::rc::Rc;

use envoy::extension;
use envoy::extension::{ConfigStatus, InstanceId, Result};
use envoy::host::{http::client as http_client, stats, time};

use super::config::SampleHttpFilterConfig;
use super::filter::SampleHttpFilter;
use super::stats::SampleHttpFilterStats;

/// Factory for creating Sample HTTP Filter instances
/// (one filter instance per HTTP request).
pub struct SampleHttpFilterFactory<'a> {
    // This example shows how multiple filter instances could share
    // the same configuration.
    config: Rc<SampleHttpFilterConfig>,
    // This example shows how multiple filter instances could share
    // metrics.
    stats: Rc<SampleHttpFilterStats>,
    // This example shows how to use Time API, HTTP Client API and
    // Metrics API provided by Envoy host.
    time_service: &'a dyn time::Service,
    http_client: &'a dyn http_client::Client,
}

impl<'a> SampleHttpFilterFactory<'a> {
    /// Creates a new factory.
    pub fn new(
        time_service: &'a dyn time::Service,
        http_client: &'a dyn http_client::Client,
        metrics_service: &'a dyn stats::Service,
    ) -> Result<Self> {
        let stats = SampleHttpFilterStats::new(
            metrics_service.counter("examples.http_filter.requests_total")?,
            metrics_service.gauge("examples.http_filter.requests_active")?,
            metrics_service.histogram("examples.http_filter.response_body_size_bytes")?,
        );
        // Inject dependencies on Envoy host APIs
        Ok(SampleHttpFilterFactory {
            config: Rc::new(SampleHttpFilterConfig::default()),
            stats: Rc::new(stats),
            time_service,
            http_client,
        })
    }

    /// Creates a new factory bound to the actual Envoy ABI.
    pub fn default() -> Result<Self> {
        Self::new(
            time::Service::default(),
            http_client::Client::default(),
            stats::Service::default(),
        )
    }
}

impl<'a> extension::Factory for SampleHttpFilterFactory<'a> {
    type Extension = SampleHttpFilter<'a>;

    /// The reference name for Sample HTTP Filter.
    ///
    /// This name appears in `Envoy` configuration as a value of `root_id` field
    /// (also known as `group_name`).
    const NAME: &'static str = "examples.http_filter";

    /// Is called when Envoy creates a new Listener that uses Sample HTTP Filter.
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        ops: &dyn extension::factory::ConfigureOps,
    ) -> Result<ConfigStatus> {
        let config = match ops.get_configuration()? {
            Some(bytes) => SampleHttpFilterConfig::try_from(bytes.as_slice())?,
            None => SampleHttpFilterConfig::default(),
        };
        self.config = Rc::new(config);
        Ok(ConfigStatus::Accepted)
    }

    /// Is called to create a unique instance of Sample HTTP Filter
    /// for each HTTP request.
    fn new_extension(&mut self, instance_id: InstanceId) -> Result<Self::Extension> {
        Ok(SampleHttpFilter::new(
            Rc::clone(&self.config),
            Rc::clone(&self.stats),
            instance_id,
            self.time_service,
            self.http_client,
        ))
    }
}
