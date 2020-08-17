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

//! `Envoy` `Access Logger` APIs for use in unit tests.

use std::marker::PhantomData;

use envoy::extension::access_logger;
use envoy::extension::{self, AccessLogger, ConfigStatus, DrainStatus};
use envoy::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};
use envoy::host::Bytes;

/// Reference to an `Access Logger` extension.
pub(crate) struct DynAccessLogger<'a, L> {
    logger: L,
    phantom: PhantomData<&'a L>,
}

impl<'a, L> DynAccessLogger<'a, L>
where
    L: AccessLogger,
{
    pub fn wrap(logger: L) -> Self {
        Self {
            logger,
            phantom: PhantomData,
        }
    }
}

impl<'a, L> AccessLogger for DynAccessLogger<'a, L>
where
    L: AccessLogger,
{
    fn name() -> &'static str {
        L::name()
    }

    fn on_configure(
        &mut self,
        config: Bytes,
        ops: &dyn access_logger::ConfigureOps,
    ) -> extension::Result<ConfigStatus> {
        self.logger.on_configure(config, ops)
    }

    fn on_drain(&mut self) -> extension::Result<DrainStatus> {
        self.logger.on_drain()
    }

    fn on_log(&mut self, ops: &dyn access_logger::LogOps) -> extension::Result<()> {
        self.logger.on_log(ops)
    }

    fn on_http_call_response(
        &mut self,
        request_id: HttpClientRequestHandle,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
        http_client_ops: &dyn HttpClientResponseOps,
    ) -> extension::Result<()> {
        self.logger.on_http_call_response(
            request_id,
            num_headers,
            body_size,
            num_trailers,
            http_client_ops,
        )
    }
}
