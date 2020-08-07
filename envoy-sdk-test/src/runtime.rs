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

use std::marker::PhantomData;

use envoy::extension::factory::{self, ConfigStatus, DrainStatus};
use envoy::extension::filter::network;
use envoy::extension::{self, ExtensionFactory, HttpFilter, InstanceId, NetworkFilter};
use envoy::host::{self, Bytes};

use crate::host::{FakeClock, FakeHttpClient, FakeStats};

#[derive(Default)]
pub struct FakeEnvoy {
    pub http_client: FakeHttpClient,
    pub stats: FakeStats,
    pub clock: FakeClock,
}

pub struct FakeTcpListener<'a> {
    _envoy: &'a FakeEnvoy,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn NetworkFilter + 'a>> + 'a>>,
}

pub struct FakeTcpConnection<'a, 'b> {
    _listener: &'b FakeTcpListener<'a>,
    filter: Option<Box<dyn NetworkFilter + 'a>>,
    state: FakeTcpConnectionState,
}

#[derive(Debug, Default)]
struct FakeTcpConnectionState {
    read_buffer: Vec<u8>,
    _write_buffer: Vec<u8>,
}

pub struct FakeListenerBuilder<'a> {
    envoy: &'a FakeEnvoy,
}

pub struct FakeTcpListenerBuilder<'a> {
    listener: FakeListenerBuilder<'a>,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn NetworkFilter + 'a>> + 'a>>,
}

impl FakeEnvoy {
    pub fn listener(&self) -> FakeListenerBuilder {
        FakeListenerBuilder { envoy: self }
    }
}

impl<'a> FakeTcpListener<'a> {
    pub fn new_connection<'b>(&'b mut self) -> extension::Result<FakeTcpConnection<'a, 'b>> {
        let filter = match &mut self.filter_factory {
            Some(filter_factory) => Some(filter_factory.new_extension(InstanceId::from(0))?), // TODO: proper instance id
            None => None,
        };
        Ok(FakeTcpConnection {
            _listener: self,
            filter,
            state: FakeTcpConnectionState::default(),
        })
    }
}

impl network::DownstreamDataOps for FakeTcpConnectionState {
    fn downstream_data(&self, _offset: usize, _max_size: usize) -> host::Result<Bytes> {
        Ok(Bytes::default())
    }
}

impl<'a, 'b> FakeTcpConnection<'a, 'b> {
    pub fn open(&mut self) -> extension::Result<network::FilterStatus> {
        match &mut self.filter {
            Some(filter) => filter.on_new_connection(),
            None => Ok(network::FilterStatus::Continue),
        }
    }

    pub fn send(&mut self, data: &[u8]) -> extension::Result<network::FilterStatus> {
        self.state.read_buffer.extend(data);
        match &mut self.filter {
            Some(filter) => {
                filter.on_downstream_data(self.state.read_buffer.len(), false, &self.state)
            }
            None => Ok(network::FilterStatus::Continue),
        }
    }
}

impl<'a> FakeListenerBuilder<'a> {
    pub fn tcp(self) -> FakeTcpListenerBuilder<'a> {
        FakeTcpListenerBuilder {
            listener: self,
            filter_factory: None,
        }
    }
}

impl<'a> FakeTcpListenerBuilder<'a> {
    pub fn network_filter<T>(mut self, filter_factory: T) -> extension::Result<Self>
    where
        T: ExtensionFactory + 'a,
        T::Extension: NetworkFilter,
    {
        self.filter_factory = Some(Box::new(DynNetworkFilterFactory::wrap(filter_factory)));
        Ok(self)
    }

    pub fn configure<C>(self, config: C) -> extension::Result<FakeTcpListener<'a>>
    where
        C: Into<Vec<u8>>,
    {
        let filter_factory = match self.filter_factory {
            Some(mut filter_factory) => {
                filter_factory.on_configure(Bytes::from(config.into()), &NoOps)?;
                Some(filter_factory)
            }
            None => None,
        };
        Ok(FakeTcpListener {
            _envoy: self.listener.envoy,
            filter_factory,
        })
    }
}

struct DynNetworkFilterFactory<'a, F> {
    factory: F,
    phantom: PhantomData<&'a F>,
}

impl<'a, F> DynNetworkFilterFactory<'a, F>
where
    F: ExtensionFactory,
    F::Extension: NetworkFilter,
{
    fn wrap(factory: F) -> Self {
        Self {
            factory,
            phantom: PhantomData,
        }
    }
}

impl<'a, F> ExtensionFactory for DynNetworkFilterFactory<'a, F>
where
    F: ExtensionFactory,
    F::Extension: NetworkFilter,
{
    type Extension = Box<dyn NetworkFilter + 'a>;

    fn name() -> &'static str {
        F::name()
    }

    fn on_configure(
        &mut self,
        config: Bytes,
        ops: &dyn factory::ConfigureOps,
    ) -> extension::Result<ConfigStatus> {
        self.factory.on_configure(config, ops)
    }

    fn new_extension(&mut self, instance_id: InstanceId) -> extension::Result<Self::Extension> {
        self.factory
            .new_extension(instance_id)
            .map(|filter| Box::new(filter) as Box<dyn NetworkFilter>)
    }

    fn on_drain(&mut self) -> extension::Result<DrainStatus> {
        self.factory.on_drain()
    }
}

struct DynHttpFilterFactory<F> {
    factory: F,
}

impl<F> ExtensionFactory for DynHttpFilterFactory<F>
where
    F: ExtensionFactory + 'static,
    F::Extension: HttpFilter,
{
    type Extension = Box<dyn HttpFilter>;

    fn name() -> &'static str {
        F::name()
    }

    fn on_configure(
        &mut self,
        config: Bytes,
        ops: &dyn factory::ConfigureOps,
    ) -> extension::Result<ConfigStatus> {
        self.factory.on_configure(config, ops)
    }

    fn new_extension(&mut self, instance_id: InstanceId) -> extension::Result<Self::Extension> {
        self.factory
            .new_extension(instance_id)
            .map(|filter| Box::new(filter) as Box<dyn HttpFilter>)
    }

    fn on_drain(&mut self) -> extension::Result<DrainStatus> {
        self.factory.on_drain()
    }
}

struct NoOps;
impl factory::ConfigureOps for NoOps {}
