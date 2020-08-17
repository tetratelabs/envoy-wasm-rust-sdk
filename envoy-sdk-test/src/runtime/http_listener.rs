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

//! Fake `Envoy` environment for `HTTP`-level extensions.

use envoy::extension::{self, factory, ExtensionFactory, HttpFilter, InstanceId};
use envoy::host::{Bytes, HeaderMap};

use super::{FakeEnvoy, FakeListenerBuilder};
use crate::extension::filter::http::DynHttpFilterFactory;

/// Factory of fake `Envoy` `Listeners` for testing `HTTP`-level extensions.
pub struct FakeHttpListenerBuilder<'a> {
    listener: FakeListenerBuilder<'a>,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn HttpFilter + 'a>> + 'a>>,
}

/// Fake `Envoy` `Listener` for testing `Http Filter` extensions.
pub struct FakeHttpListener<'a> {
    _envoy: &'a FakeEnvoy,
    filter_factory: Option<Box<dyn ExtensionFactory<Extension = Box<dyn HttpFilter + 'a>> + 'a>>,
}

/// Fake `Envoy` `HTTP Stream` for testing `Http Filter` extensions.
pub struct FakeHttpStream<'a, 'b> {
    _listener: &'b FakeHttpListener<'a>,
    _filter: Option<Box<dyn HttpFilter + 'a>>,
    _state: FakeHttpStreamState,
    _downstream: FakeHttpDownstream,
    _upstream: FakeHttpUpstream,
}

/// Encapsulates state of a fake `TCP` connection.
#[derive(Debug, Default)]
struct FakeHttpStreamState {}

/// Encapsulates state of the implicit `Envoy -> Upstream` interactions.
#[derive(Debug, Default)]
pub struct FakeHttpDownstream {
    received_headers: HeaderMap,
    received_bytes: Vec<u8>,
    received_trailers: HeaderMap,
    received_end: bool,
}

/// Encapsulates state of the implicit `Downstream <- Envoy` interactions.
#[derive(Debug, Default)]
pub struct FakeHttpUpstream {
    received_headers: HeaderMap,
    received_bytes: Vec<u8>,
    received_trailers: HeaderMap,
    received_end: bool,
}

impl<'a> FakeHttpListenerBuilder<'a> {
    pub(super) fn new(listener: FakeListenerBuilder<'a>) -> Self {
        FakeHttpListenerBuilder {
            listener,
            filter_factory: None,
        }
    }

    /// Adds a given `HttpFilter` extension to the fake `Envoy` `Listener`.
    pub fn http_filter<T>(mut self, filter_factory: T) -> Self
    where
        T: ExtensionFactory + 'a,
        T::Extension: HttpFilter,
    {
        self.filter_factory = Some(Box::new(DynHttpFilterFactory::wrap(filter_factory)));
        self
    }

    /// Finishes building a fake `Envoy` `Listener` by applying a given configuration to
    /// the `HttpFilter` extension.
    pub fn configure<C>(self, config: C) -> extension::Result<FakeHttpListener<'a>>
    where
        C: AsRef<[u8]>,
    {
        let filter_factory = match self.filter_factory {
            Some(mut filter_factory) => {
                filter_factory.on_configure(Bytes::from(config.as_ref().to_owned()), &NoOps)?;
                Some(filter_factory)
            }
            None => None,
        };
        Ok(FakeHttpListener {
            _envoy: self.listener.envoy,
            filter_factory,
        })
    }
}

impl<'a> FakeHttpListener<'a> {
    /// Returns a new `HTTP Stream`.
    pub fn new_http_stream<'b>(&'b mut self) -> extension::Result<FakeHttpStream<'a, 'b>> {
        let filter = match &mut self.filter_factory {
            Some(filter_factory) => Some(filter_factory.new_extension(InstanceId::from(0))?), // TODO: proper instance id
            None => None,
        };
        Ok(FakeHttpStream {
            _listener: self,
            _filter: filter,
            _state: FakeHttpStreamState::default(),
            _downstream: FakeHttpDownstream::default(),
            _upstream: FakeHttpUpstream::default(),
        })
    }
}

struct NoOps;

impl factory::ConfigureOps for NoOps {}
