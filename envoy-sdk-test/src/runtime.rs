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

use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;

use envoy::extension::factory::{self, ConfigStatus, DrainStatus};
use envoy::extension::filter::network::{self, CloseType};
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
    downstream: FakeTcpDownstream,
    upstream: FakeTcpUpstream,
}

#[derive(Debug, Default)]
struct FakeTcpConnectionState {
    received_connect: bool,
    downstream_read_buffer: RefCell<Vec<u8>>,
    downstream_read_end_of_stream: bool,
    downstream_read_paused: bool,
    downstream_write_buffer: RefCell<Vec<u8>>,
    downstream_write_end_of_stream: bool,
    upstream_read_buffer: RefCell<Vec<u8>>,
    upstream_read_end_of_stream: bool,
    upstream_read_paused: bool,
}

#[derive(Debug, Default)]
pub struct FakeTcpUpstream {
    received_connect: bool,
    received_buffer: Vec<u8>,
    received_close: bool,
}

impl FakeTcpUpstream {
    pub fn received_connect(&self) -> bool {
        self.received_connect
    }

    pub fn received_close(&self) -> bool {
        self.received_close
    }

    pub fn drain_received_bytes(&mut self) -> Vec<u8> {
        self.received_buffer.drain(..).collect()
    }

    fn receive_connect(&mut self) {
        assert_eq!(self.received_connect, false);
        self.received_connect = true;
    }

    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_close, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the upstream after closing connection");
        self.received_close |= end_of_stream;
        self.received_buffer.extend(data);
    }
}

#[derive(Debug, Default)]
pub struct FakeTcpDownstream {
    received_buffer: Vec<u8>,
    received_close: bool,
}

impl FakeTcpDownstream {
    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_close, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the downstream after closing connection");
        self.received_close |= end_of_stream;
        self.received_buffer.extend(data);
    }

    pub fn received_close(&self) -> bool {
        self.received_close
    }

    pub fn drain_received_bytes(&mut self) -> Vec<u8> {
        self.received_buffer.drain(..).collect()
    }
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
            downstream: FakeTcpDownstream::default(),
            upstream: FakeTcpUpstream::default(),
        })
    }
}

mod proxy_wasm_mock {
    use std::cmp;

    use envoy::host::{self, Bytes};

    /// Reads buffer similarly to `Proxy Wasm` inside Envoy.
    pub fn get_buffer_bytes(buf: &[u8], offset: usize, max_size: usize) -> host::Result<Bytes> {
        // implementation based on `proxy-wasm/proxy-wasm-cpp-host`

        // Check for overflow.
        if let (_, true) = offset.overflowing_add(max_size) {
            return Err("Status::BadArgument".into());
        }
        let max_size = cmp::min(max_size, buf.len() - offset);
        if max_size > 0 {
            return Ok(buf[offset..offset + max_size].to_owned().into());
        }
        Ok(Bytes::default())
    }

    /// Mutates buffer similarly to `Proxy Wasm` inside Envoy.
    pub fn set_buffer_bytes(
        buf: &mut Vec<u8>,
        start: usize,
        length: usize,
        data: &[u8],
    ) -> host::Result<()> {
        // implementation based on `envoyproxy/envoy-wasm`

        if start == 0 {
            if length == 0 {
                let tail: Vec<u8> = buf.drain(..).collect();
                buf.extend(data);
                buf.extend(tail);
                Ok(())
            } else if length >= buf.len() {
                buf.truncate(0);
                buf.extend(data);
                Ok(())
            } else {
                Err("WasmResult::BadArgument".into())
            }
        } else if start >= buf.len() {
            buf.extend(data);
            Ok(())
        } else {
            Err("WasmResult::BadArgument".into())
        }
    }
}

impl network::DownstreamDataOps for FakeTcpConnectionState {
    fn downstream_data(&self, offset: usize, max_size: usize) -> host::Result<Bytes> {
        let buf = self.downstream_read_buffer.borrow();
        proxy_wasm_mock::get_buffer_bytes(&buf, offset, max_size)
    }

    fn mutate_downstream_data(&self, action: network::BufferAction) -> host::Result<()> {
        action.execute(|start: usize, length: usize, data: &[u8]| {
            let mut buf = self.downstream_read_buffer.borrow_mut();
            proxy_wasm_mock::set_buffer_bytes(&mut *buf, start, length, data)
        })
    }
}

impl network::UpstreamDataOps for FakeTcpConnectionState {
    fn upstream_data(&self, offset: usize, max_size: usize) -> host::Result<Bytes> {
        let buf = self.upstream_read_buffer.borrow();
        proxy_wasm_mock::get_buffer_bytes(&buf, offset, max_size)
    }

    fn mutate_upstream_data(&self, action: network::BufferAction) -> host::Result<()> {
        action.execute(|start: usize, length: usize, data: &[u8]| {
            let mut buf = self.upstream_read_buffer.borrow_mut();
            proxy_wasm_mock::set_buffer_bytes(&mut *buf, start, length, data)
        })
    }
}

impl network::DownstreamCloseOps for FakeTcpConnectionState {}

impl network::UpstreamCloseOps for FakeTcpConnectionState {}

impl<'a, 'b> FakeTcpConnection<'a, 'b> {
    pub fn downstream(&mut self) -> &mut FakeTcpDownstream {
        &mut self.downstream
    }

    pub fn upstream(&mut self) -> &mut FakeTcpUpstream {
        &mut self.upstream
    }

    pub fn downstream_read_buffer(&self) -> Ref<'_, Vec<u8>> {
        self.state.downstream_read_buffer.borrow()
    }

    pub fn downstream_read_buffer_mut(&mut self) -> RefMut<'_, Vec<u8>> {
        self.state.downstream_read_buffer.borrow_mut()
    }

    pub fn upstream_read_buffer(&self) -> Ref<'_, Vec<u8>> {
        self.state.upstream_read_buffer.borrow()
    }

    pub fn upstream_read_buffer_mut(&mut self) -> RefMut<'_, Vec<u8>> {
        self.state.upstream_read_buffer.borrow_mut()
    }

    pub fn drain_downstream_write_buffer(&mut self) -> Vec<u8> {
        self.state
            .downstream_write_buffer
            .borrow_mut()
            .drain(..)
            .collect()
    }

    pub fn receive_connect_from_downstream(&mut self) -> extension::Result<network::FilterStatus> {
        assert_eq!(self.state.received_connect, false, "unit test is trying to do something that actual Envoy would never do: don't open connection for the second time");
        self.state.received_connect = true;
        let status = match &mut self.filter {
            Some(filter) => filter.on_new_connection(),
            None => Ok(network::FilterStatus::Continue),
        };
        match status {
            Ok(network::FilterStatus::Continue) => {
                self.upstream.receive_connect();
            }
            Ok(network::FilterStatus::StopIteration) => {
                self.state.downstream_read_paused = true;
            }
            Ok(status) => panic!(
                "oops, it seems that test framework got outdated: unexpected status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    pub fn receive_data_from_downstream(
        &mut self,
        data: &[u8],
    ) -> extension::Result<network::FilterStatus> {
        self.do_receive_data_from_downstream(data, false)
    }

    pub fn receive_close_from_downstream(&mut self) -> extension::Result<network::FilterStatus> {
        let status = self.do_receive_data_from_downstream(&[], true)?;
        if let Some(filter) = &mut self.filter {
            filter.on_downstream_close(CloseType::Remote, &self.state)?;
        }
        Ok(status)
    }

    fn do_receive_data_from_downstream(
        &mut self,
        data: &[u8],
        end_of_stream: bool,
    ) -> extension::Result<network::FilterStatus> {
        if !self.state.received_connect {
            let status = self.receive_connect_from_downstream();
            match status {
                Ok(network::FilterStatus::Continue) => (),
                status => return status,
            };
        }
        assert_eq!(self.state.downstream_read_end_of_stream, false, "unit test is trying to do something that actual Envoy would never do: downstream cannot keep sending data after closing the connection");
        self.state.downstream_read_buffer.borrow_mut().extend(data);
        self.state.downstream_read_end_of_stream |= end_of_stream;
        if self.state.downstream_read_paused {
            return Ok(network::FilterStatus::StopIteration);
        }
        let status = match &mut self.filter {
            Some(filter) => {
                let buf_len = self.state.downstream_read_buffer.borrow().len();
                filter.on_downstream_data(buf_len, end_of_stream, &self.state)
            }
            None => Ok(network::FilterStatus::Continue),
        };
        match &status {
            Ok(network::FilterStatus::Continue) => {
                self.upstream.receive_data(
                    self.state
                        .downstream_read_buffer
                        .borrow_mut()
                        .drain(..)
                        .collect(),
                    end_of_stream,
                );
            }
            Ok(network::FilterStatus::StopIteration) => {
                self.state.downstream_read_paused = true;
            }
            Ok(status) => panic!(
                "oops, it seems that test framework got outdated: unexpected status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    pub fn receive_data_from_upstream(
        &mut self,
        data: &[u8],
    ) -> extension::Result<network::FilterStatus> {
        self.do_receive_data_from_upstream(data, false)
    }

    pub fn receive_close_from_upstream(&mut self) -> extension::Result<network::FilterStatus> {
        let status = self.do_receive_data_from_upstream(&[], true)?;
        if let Some(filter) = &mut self.filter {
            // CloseType::Unknown to simulate current behaviour of `envoyproxy/envoy-wasm`
            filter.on_upstream_close(CloseType::Unknown, &self.state)?;
        }
        Ok(status)
    }

    pub fn do_receive_data_from_upstream(
        &mut self,
        data: &[u8],
        end_of_stream: bool,
    ) -> extension::Result<network::FilterStatus> {
        assert_eq!(self.upstream.received_connect, true, "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving a connect");
        assert_eq!(self.state.upstream_read_end_of_stream, false, "unit test is trying to do something that actual Envoy would never do: upstream cannot keep sending data after closing the connection");
        self.state.upstream_read_buffer.borrow_mut().extend(data);
        self.state.upstream_read_end_of_stream |= end_of_stream;
        if self.state.upstream_read_paused {
            return Ok(network::FilterStatus::StopIteration);
        }
        let status = match &mut self.filter {
            Some(filter) => filter.on_upstream_data(data.len(), end_of_stream, &self.state),
            None => Ok(network::FilterStatus::Continue),
        };
        match &status {
            Ok(network::FilterStatus::Continue) => {
                self.downstream.receive_data(
                    self.state
                        .upstream_read_buffer
                        .borrow_mut()
                        .drain(..)
                        .collect(),
                    end_of_stream,
                );
            }
            Ok(network::FilterStatus::StopIteration) => {
                self.state.upstream_read_paused = true;
            }
            Ok(status) => panic!(
                "oops, it seems that test framework got outdated: unexpected status {:?}",
                status
            ),
            _ => (),
        };
        Ok(network::FilterStatus::Continue)
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
