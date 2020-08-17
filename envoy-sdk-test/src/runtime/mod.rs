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

//! Fake `Envoy` environment for use in unit tests.
//!
//! Most importantly, it attempts to reproduce (to a reasonable degree)
//! the original request processing flow of `Envoy`, including such notions
//! as `Downstream`, `Upstream`, `Read Buffer`, etc.
//!
//! This way, instead of making assumptions about how `Envoy` would iteract with
//! your extension at runtime, you can rely on the test environment for that.
//!
//! Overall, fake `Envoy` environment lets you focus in your tests on the
//! observed behaviour from the perspective of the `Downstream` and the `Upstream`
//! instead of focusing merely on the mechanics of `Envoy <=> Wasm` interaction.

use std::cell::RefCell;

use envoy::extension::InstanceId;

use crate::host::{FakeClock, FakeHttpClient, FakeStats};

pub use self::access_log::{FakeAccessLog, FakeAccessLogBuilder};
pub use self::http_listener::{FakeHttpListener, FakeHttpListenerBuilder};
pub use self::tcp_listener::{FakeTcpListener, FakeTcpListenerBuilder};

mod access_log;
mod http_listener;
mod tcp_listener;

/// Fake `Envoy` environment to run unit tests in.
#[derive(Default)]
pub struct FakeEnvoy {
    /// Fake `HTTP Client API`.
    pub http_client: FakeHttpClient,
    /// Fake `Stats API`.
    pub stats: FakeStats,
    /// Fake `Clock API`.
    pub clock: FakeClock,

    /// Fake id generator.
    generator: FakeIdGenerator,
}

/// Fake id generator.
#[derive(Default)]
struct FakeIdGenerator {
    next_instance_id: RefCell<u32>,
}

/// Factory of fake `Envoy` `Listeners`.
pub struct FakeListenerBuilder<'a> {
    envoy: &'a FakeEnvoy,
}

impl FakeEnvoy {
    /// Returns a factory for building a fake `Envoy` `Listener`.
    pub fn listener(&self) -> FakeListenerBuilder<'_> {
        FakeListenerBuilder::new(self)
    }

    /// Returns a factory for building a fake `Envoy` `Access Log`.
    pub fn access_log(&self) -> FakeAccessLogBuilder<'_> {
        FakeAccessLogBuilder::new(self)
    }
}

impl<'a> FakeListenerBuilder<'a> {
    pub(super) fn new(envoy: &'a FakeEnvoy) -> Self {
        FakeListenerBuilder { envoy }
    }

    /// Returns a factory for building a fake `Envoy` `Listener` with `TCP`-level extensions.
    pub fn tcp(self) -> FakeTcpListenerBuilder<'a> {
        FakeTcpListenerBuilder::new(self)
    }

    /// Returns a factory for building a fake `Envoy` `Listener` with `HTTP`-level extensions.
    pub fn http(self) -> FakeHttpListenerBuilder<'a> {
        FakeHttpListenerBuilder::new(self)
    }
}

impl FakeIdGenerator {
    /// Returns a unique id for a new extension instance.
    fn new_instance_id(&self) -> InstanceId {
        let next = *self.next_instance_id.borrow();
        *self.next_instance_id.borrow_mut() += 1;
        next.into()
    }
}
