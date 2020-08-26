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

//! Fake `Envoy` environment for `TCP`-level extensions.

use std::cell::RefCell;

use envoy::extension::factory;
use envoy::extension::filter::network::{self, PeerType};
use envoy::extension::{self, ExtensionFactory, NetworkFilter};
use envoy::host::buffer::{internal::TransformExecutor, Transform};
use envoy::host::http::client::HttpClientRequestHandle;
use envoy::host::{self, ByteString};

use super::{FakeEnvoy, FakeListenerBuilder};
use crate::extension::filter::network::DynNetworkFilterFactory;
use crate::host::simulate;
use crate::host::FakeHttpClientResponse;

/// Factory of fake `Envoy` `Listeners` for testing `TCP`-level extensions.
pub struct FakeTcpListenerBuilder<'a> {
    listener: FakeListenerBuilder<'a>,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn NetworkFilter + 'a>> + 'a>>,
}

/// Fake `Envoy` `Listener` for testing `Network Filter` extensions.
pub struct FakeTcpListener<'a> {
    envoy: &'a FakeEnvoy,
    filter_factory: Box<dyn ExtensionFactory<Extension = Box<dyn NetworkFilter + 'a>> + 'a>,
}

/// Fake `Envoy` `Connection` for testing `Network Filter` extensions.
pub struct FakeTcpConnection<'a> {
    filter: Box<dyn NetworkFilter + 'a>,
    state: FakeTcpConnectionState,
    downstream: FakeTcpDownstream,
    upstream: FakeTcpUpstream,
}

/// Encapsulates state of a fake `TCP` connection.
#[derive(Debug, Default)]
struct FakeTcpConnectionState {
    received_connect: bool,
    downstream_read_buffer: RefCell<Vec<u8>>,
    downstream_read_end_of_stream: bool,
    // unlike with `downstream_read_buffer`, this buffer is only valid for the duration of
    // `on_upstream_data` callback (quirk of Envoy)
    upstream_read_buffer: RefCell<Vec<u8>>,
    upstream_read_end_of_stream: bool,
}

/// Encapsulates state of the implicit `Envoy -> Upstream` interactions.
#[derive(Debug, Default)]
pub struct FakeTcpUpstream {
    received_connect: bool,
    received_bytes: Vec<u8>,
    received_close: bool,
}

/// Encapsulates state of the implicit `Downstream <- Envoy` interactions.
#[derive(Debug, Default)]
pub struct FakeTcpDownstream {
    received_bytes: Vec<u8>,
    received_close: bool,
}

impl<'a> FakeTcpListenerBuilder<'a> {
    pub(super) fn new(listener: FakeListenerBuilder<'a>) -> Self {
        FakeTcpListenerBuilder {
            listener,
            filter_factory: None,
        }
    }

    /// Adds a given `NetworkFilter` extension to the fake `Envoy` `Listener`.
    pub fn network_filter<T>(mut self, filter_factory: T) -> Self
    where
        T: ExtensionFactory + 'a,
        T::Extension: NetworkFilter,
    {
        self.filter_factory = Some(Box::new(DynNetworkFilterFactory::wrap(filter_factory)));
        self
    }

    /// Finishes building a fake `Envoy` `Listener` by applying a given configuration to
    /// the `NetworkFilter` extension.
    pub fn configure<C>(self, config: C) -> extension::Result<FakeTcpListener<'a>>
    where
        C: AsRef<[u8]>,
    {
        let mut filter_factory = self.filter_factory.expect(
            "Network Filter extension factory must be added prior to calling `configure(...)`",
        );
        filter_factory.on_configure(config.as_ref().into(), &NoOps)?;
        Ok(FakeTcpListener {
            envoy: self.listener.envoy,
            filter_factory,
        })
    }
}

impl<'a> FakeTcpListener<'a> {
    /// Returns a new `TCP` connection.
    pub fn new_connection(&mut self) -> extension::Result<FakeTcpConnection<'a>> {
        let filter = self
            .filter_factory
            .new_extension(self.envoy.generator.new_instance_id())?;
        Ok(FakeTcpConnection {
            filter,
            state: FakeTcpConnectionState::default(),
            downstream: FakeTcpDownstream::default(),
            upstream: FakeTcpUpstream::default(),
        })
    }
}

impl FakeTcpDownstream {
    /// Simulates `Downstream <- Envoy` response data.
    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_close, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the downstream after closing connection");
        self.received_close |= end_of_stream;
        self.received_bytes.extend(data);
    }

    /// Returns `true` after `Downstream -> Envoy` connection has been closed on `Envoy` side.
    pub fn received_close(&self) -> bool {
        self.received_close
    }

    /// Returns response data received from `Envoy` since the last call to this method.
    pub fn drain_received_bytes(&mut self) -> Vec<u8> {
        self.received_bytes.drain(..).collect()
    }
}

impl FakeTcpUpstream {
    /// Simulates a new `Envoy -> Upstream` connection.
    fn receive_connect(&mut self) {
        assert_eq!(self.received_connect, false, "unit test is trying to do something that actual Envoy would never do: don't try to open connection for the second time");
        self.received_connect = true;
    }

    /// Simulates `Envoy -> Upstream` request data.
    fn receive_data(&mut self, data: Vec<u8>, end_of_stream: bool) {
        assert_eq!(self.received_connect, true, "unit test is trying to do something that actual Envoy would never do: don't try to send data to the upstream without opening the connection first");
        assert_eq!(self.received_close, false, "unit test is trying to do something that actual Envoy would never do: don't keep sending data to the upstream after closing connection");
        self.received_close |= end_of_stream;
        self.received_bytes.extend(data);
    }

    /// Returns `true` after `Envoy -> Upstream` connection has been opened.
    pub fn received_connect(&self) -> bool {
        self.received_connect
    }

    /// Returns `true` after `Envoy -> Upstream` connection has been closed on `Envoy` side.
    pub fn received_close(&self) -> bool {
        self.received_close
    }

    /// Returns request data received from `Envoy` since the last call to this method.
    pub fn drain_received_bytes(&mut self) -> Vec<u8> {
        self.received_bytes.drain(..).collect()
    }
}

impl<'a> FakeTcpConnection<'a> {
    /// Simulate `Downstream -> Envoy` connect.
    pub fn simulate_connect_from_downstream(&mut self) -> extension::Result<network::FilterStatus> {
        assert_eq!(self.state.received_connect, false, "unit test is trying to do something that actual Envoy would never do: don't connect for the second time");
        self.state.received_connect = true;
        let status = self.filter.on_new_connection();
        match status {
            Ok(network::FilterStatus::Continue) => {
                self.upstream.receive_connect();
            }
            Ok(network::FilterStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Downstream -> Envoy` data.
    pub fn simulate_data_from_downstream(
        &mut self,
        data: &[u8],
    ) -> extension::Result<network::FilterStatus> {
        self.receive_data_from_downstream(data, false)
    }

    /// Simulate `Downstream -> Envoy` close of connection.
    pub fn simulate_close_from_downstream(&mut self) -> extension::Result<network::FilterStatus> {
        let status = self.receive_data_from_downstream(&[], true)?;
        self.filter
            .on_downstream_close(PeerType::Remote, &self.state)?;
        Ok(status)
    }

    fn receive_data_from_downstream(
        &mut self,
        data: &[u8],
        end_of_stream: bool,
    ) -> extension::Result<network::FilterStatus> {
        if !self.state.received_connect {
            let status = self.simulate_connect_from_downstream();
            match status {
                Ok(network::FilterStatus::Continue) => (),
                status => return status,
            };
        }
        assert_eq!(self.state.downstream_read_end_of_stream, false, "unit test is trying to do something that actual Envoy would never do: downstream cannot keep sending data after closing the connection");
        self.state.downstream_read_buffer.borrow_mut().extend(data);
        self.state.downstream_read_end_of_stream |= end_of_stream;
        // notice that Envoy doesn't memorize what status a Network Filter returned last time;
        // that is why `on_downstream_data` callback will typically be called on every data receival
        // even if the filter previously returned `StopIteration` and hasn't called `resume()` after that.
        // There is a case where Envoy doesn't call `on_downstream_data` - it happens when `connection::readDisable(true)`
        // is called.
        // In a typical TCP scenario with `tcp_proxy` filter involved, `connection::readDisable(true)` is called
        // during filter chain construction and has effect until `onNewConnection()` is called on `tcp_proxy`
        // (and even later after that). Which means that if a Filter returns StopIteration from `on_new_connection`,
        // `tcp_proxy` won't be called until `resume` is called first.
        let status = {
            let buf_len = self.state.downstream_read_buffer.borrow().len();
            self.filter
                .on_downstream_data(buf_len, end_of_stream, &self.state)
        };
        match status {
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
            Ok(network::FilterStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        status
    }

    /// Simulate `Envoy <- Upstream` data.
    pub fn simulate_data_from_upstream(
        &mut self,
        data: &[u8],
    ) -> extension::Result<network::FilterStatus> {
        self.receive_data_from_upstream(data, false)
    }

    /// Simulate `Envoy <- Upstream` close of connection.
    pub fn simulate_close_from_upstream(&mut self) -> extension::Result<network::FilterStatus> {
        let status = self.receive_data_from_upstream(&[], true)?;
        // use CloseType::Unknown to simulate the exact behaviour of `envoyproxy/envoy-wasm`
        self.filter
            .on_upstream_close(PeerType::Unknown, &self.state)?;
        Ok(status)
    }

    fn receive_data_from_upstream(
        &mut self,
        data: &[u8],
        end_of_stream: bool,
    ) -> extension::Result<network::FilterStatus> {
        assert_eq!(self.upstream.received_connect, true, "unit test is trying to do something that actual Envoy would never do: upstream cannot respond prior to receiving a connect");
        assert_eq!(self.state.upstream_read_end_of_stream, false, "unit test is trying to do something that actual Envoy would never do: upstream cannot keep sending data after closing the connection");
        self.state.upstream_read_buffer.replace(data.to_owned());
        // notice that Envoy doesn't memorize what status a Network Filter returned last time;
        // that is why `on_downstream_data` callback will typically be called on every data receival
        // even if the filter previously returned `StopIteration` and hasn't called `resume()` after that.
        let status = {
            let buf_len = self.state.upstream_read_buffer.borrow().len();
            self.filter
                .on_upstream_data(buf_len, end_of_stream, &self.state)
        };
        match status {
            Ok(network::FilterStatus::Continue) => {
                self.state.upstream_read_end_of_stream |= end_of_stream;
                self.downstream.receive_data(
                    self.state
                        .upstream_read_buffer
                        .borrow_mut()
                        .drain(..)
                        .collect(),
                    end_of_stream,
                );
            }
            Ok(network::FilterStatus::StopIteration) => (),
            Ok(status) => panic!(
                "oops, it seems that test framework is no longer up-to-date: unknown status {:?}",
                status
            ),
            _ => (),
        };
        // unlike with the downstream read buffer, Envoy doesn't maintain a similar buffer for the upstream.
        // if a filter chooses to return `StopIteration`, data in the buffer should be considered lost
        self.state.upstream_read_buffer.replace(Vec::new());
        status
    }

    /// Simulate a response to an HTTP request made through [`FakeHttpClient`].
    ///
    /// [`FakeHttpClient`]: ../host/http/client/struct.FakeHttpClient.html
    pub fn simulate_http_client_response(
        &mut self,
        request_id: HttpClientRequestHandle,
        response: FakeHttpClientResponse,
    ) -> extension::Result<()> {
        self.filter.on_http_call_response(
            request_id,
            response.message.headers.len(),
            response.message.body.len(),
            response.message.trailers.len(),
            &self.state,
            &response,
        )
    }

    /// Peeks into the read buffer of `Downstream -> Envoy` connection.
    pub fn peek_downstream_read_buffer(&self) -> Vec<u8> {
        self.state.downstream_read_buffer.borrow().clone()
    }

    /// Returns alleged state of the `Downstream` resulting from implicit `Downstream <- Envoy` interactions.
    pub fn downstream(&mut self) -> &mut FakeTcpDownstream {
        &mut self.downstream
    }

    /// Returns alleged state of the `Upstream` resulting from implicit `Envoy -> Upstream` interactions.
    pub fn upstream(&mut self) -> &mut FakeTcpUpstream {
        &mut self.upstream
    }
}

impl network::DownstreamDataOps for FakeTcpConnectionState {
    fn downstream_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString> {
        let buf = self.downstream_read_buffer.borrow();
        simulate::get_buffer_bytes(&buf, offset, max_size)
    }

    fn mutate_downstream_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, length: usize, data: &[u8]| {
            let mut buf = self.downstream_read_buffer.borrow_mut();
            simulate::set_buffer_bytes(&mut *buf, start, length, data)
        })
    }
}

impl network::UpstreamDataOps for FakeTcpConnectionState {
    fn upstream_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString> {
        let buf = self.upstream_read_buffer.borrow();
        simulate::get_buffer_bytes(&buf, offset, max_size)
    }

    fn mutate_upstream_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, length: usize, data: &[u8]| {
            let mut buf = self.upstream_read_buffer.borrow_mut();
            simulate::set_buffer_bytes(&mut *buf, start, length, data)
        })
    }
}

impl network::DownstreamCloseOps for FakeTcpConnectionState {}

impl network::UpstreamCloseOps for FakeTcpConnectionState {}

impl network::ConnectionCompleteOps for FakeTcpConnectionState {}

struct NoOps;

impl factory::ConfigureOps for NoOps {}
