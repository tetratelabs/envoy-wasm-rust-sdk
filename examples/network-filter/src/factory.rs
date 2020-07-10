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

use envoy::host::log::error;

use envoy::extension;
use envoy::extension::{InstanceId, Result};
use envoy::host::{http::client as http_client, stats, time};

use super::config::SampleNetworkFilterConfig;
use super::filter::SampleNetworkFilter;
use super::stats::SampleNetworkFilterStats;

/// Factory for creating sample network filter instances
/// (one filter instance per TCP connection).
pub struct SampleNetworkFilterFactory<'a> {
    // This example shows how multiple filter instances could share
    // the same configuration.
    config: Rc<SampleNetworkFilterConfig>,
    // This example shows how multiple filter instances could share
    // metrics.
    stats: Rc<SampleNetworkFilterStats>,
    // This example shows how to use Time API, HTTP Client API and
    // Metrics API provided by Envoy host.
    time_service: &'a dyn time::Service,
    http_client: &'a dyn http_client::Client,
}

impl<'a> SampleNetworkFilterFactory<'a> {
    /// Creates a new factory.
    pub fn new(
        time_service: &'a dyn time::Service,
        http_client: &'a dyn http_client::Client,
        metrics_service: &'a dyn stats::Service,
    ) -> Result<Self> {
        let stats = SampleNetworkFilterStats::new(
            metrics_service.counter("examples.network_filter.requests_total")?,
            metrics_service.gauge("examples.network_filter.requests_active")?,
            metrics_service.histogram("examples.network_filter.response_body_size_bytes")?,
        );
        // Inject dependencies on Envoy host APIs
        Ok(SampleNetworkFilterFactory {
            config: Rc::new(SampleNetworkFilterConfig::default()),
            stats: Rc::new(stats),
            time_service,
            http_client,
        })
    }

    /// Creates a new factory bound to the actual Envoy ABI.
    pub fn default() -> Result<Self> {
        SampleNetworkFilterFactory::new(
            time::Service::default(),
            http_client::Client::default(),
            stats::Service::default(),
        )
    }
}

impl<'a> extension::Factory for SampleNetworkFilterFactory<'a> {
    type Extension = SampleNetworkFilter<'a>;

    /// The reference name for sample network filter.
    ///
    /// This name appears in Envoy configuration as a value of group_name (aka, root_id) field.
    const NAME: &'static str = "examples.network_filter";

    /// Is called when Envoy creates a new Listener that uses sample network filter.
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        ops: &dyn extension::factory::ConfigureOps,
    ) -> Result<bool> {
        let config = match ops.get_configuration()? {
            Some(bytes) => match SampleNetworkFilterConfig::try_from(bytes.as_ref()) {
                Ok(value) => value,
                Err(err) => {
                    error!("failed to parse extension configuration: {}", err);
                    return Ok(false);
                }
            },
            None => SampleNetworkFilterConfig::default(),
        };
        self.config = Rc::new(config);
        Ok(true)
    }

    /// Is called to create a unique instance of sample network filter
    /// for each TCP connection.
    fn new_extension(&mut self, instance_id: InstanceId) -> Result<Self::Extension> {
        Ok(SampleNetworkFilter::new(
            Rc::clone(&self.config),
            Rc::clone(&self.stats),
            instance_id,
            self.time_service,
            self.http_client,
        ))
    }
}
