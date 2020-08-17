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

//! Fake `Envoy` environment for `Access Log` extensions.

use envoy::extension::{self, access_logger, AccessLogger};
use envoy::host::http::client::HttpClientRequestHandle;
use envoy::host::Bytes;

use super::{FakeEnvoy, FakeHttpClientResponse};
use crate::extension::access_logger::DynAccessLogger;
use crate::host::stream_info::FakeStreamInfo;

/// Fake `Envoy` `Access Log`.
pub struct FakeAccessLog<'a> {
    _envoy: &'a FakeEnvoy,
    logger: Option<Box<dyn AccessLogger + 'a>>,
}

/// Factory of a fake `Envoy` `Access Log` for testing `AccessLogger` extensions.
pub struct FakeAccessLogBuilder<'a> {
    envoy: &'a FakeEnvoy,
    logger: Option<Box<dyn AccessLogger + 'a>>,
}

impl<'a> FakeAccessLogBuilder<'a> {
    pub(super) fn new(envoy: &'a FakeEnvoy) -> Self {
        FakeAccessLogBuilder {
            envoy,
            logger: None,
        }
    }

    /// Adds a given `AccessLogger` extension to the fake `Envoy` `AccessLog`.
    pub fn logger<L>(mut self, logger: L) -> Self
    where
        L: AccessLogger + 'a,
    {
        self.logger = Some(Box::new(DynAccessLogger::wrap(logger)));
        self
    }

    /// Finishes building a fake `Envoy` `Access Log` by applying a given configuration to
    /// the `AccessLogger` extension.
    pub fn configure<C>(mut self, config: C) -> extension::Result<FakeAccessLog<'a>>
    where
        C: AsRef<[u8]>,
    {
        if let Some(ref mut logger) = self.logger {
            logger.on_configure(Bytes::from(config.as_ref().to_owned()), &NoOps)?;
        }
        Ok(FakeAccessLog {
            _envoy: self.envoy,
            logger: self.logger,
        })
    }
}

impl<'a> FakeAccessLog<'a> {
    /// Simulate log `TCP` connection.
    pub fn log_connection(&mut self, info: &FakeStreamInfo) -> extension::Result<()> {
        self.log(info)
    }

    /// Simulate log `HTTP` request.
    pub fn log_http_request(&mut self, info: &FakeStreamInfo) -> extension::Result<()> {
        self.log(info)
    }

    fn log(&mut self, info: &FakeStreamInfo) -> extension::Result<()> {
        match &mut self.logger {
            Some(logger) => logger.on_log(info),
            None => Ok(()),
        }
    }

    /// Simulate a response to an HTTP request made through [`FakeHttpClient`].
    ///
    /// [`FakeHttpClient`]: ../host/http/client/struct.FakeHttpClient.html
    pub fn simulate_http_client_response(
        &mut self,
        request_id: HttpClientRequestHandle,
        response: FakeHttpClientResponse,
    ) -> extension::Result<()> {
        match &mut self.logger {
            Some(logger) => logger.on_http_call_response(
                request_id,
                response.message.headers.len(),
                response.message.body.len(),
                response.message.trailers.len(),
                &response,
            ),
            None => Ok(()),
        }
    }
}

struct NoOps;

impl access_logger::ConfigureOps for NoOps {}
