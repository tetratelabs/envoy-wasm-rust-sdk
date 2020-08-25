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

//! Fake `Stream Info API`.

use envoy::extension::access_logger;
use envoy::host::stream_info::StreamInfo;
use envoy::host::{self, ByteString, HeaderMap};

use crate::host::http::FakeHttpMessage;

/// Represents fake `Stream Info`.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct FakeStreamInfo {
    pub connection: ConnectionInfo,

    pub http_stream: Option<RequestInfo>,
}

/// Represents TCP connection-level info.
#[derive(Debug, Default)]
pub struct ConnectionInfo {}

/// Represents HTTP request-level info.
#[derive(Debug, Default, Clone)]
pub struct RequestInfo {
    request: FakeHttpMessage,

    response: FakeHttpMessage,
}

impl StreamInfo for FakeStreamInfo {
    fn stream_property(&self, _path: &[&str]) -> host::Result<Option<ByteString>> {
        Ok(None)
    }

    fn set_stream_property(&self, _path: &[&str], _value: &[u8]) -> host::Result<()> {
        Ok(())
    }
}

impl access_logger::LogOps for FakeStreamInfo {
    fn request_headers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.request.headers.clone())
            .unwrap_or_default())
    }

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.request.headers.get(name).map(Clone::clone))
            .flatten())
    }

    fn response_headers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.response.headers.clone())
            .unwrap_or_default())
    }

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.response.headers.get(name).map(Clone::clone))
            .flatten())
    }

    fn response_trailers(&self) -> host::Result<HeaderMap> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.response.trailers.clone())
            .unwrap_or_default())
    }

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        Ok(self
            .http_stream
            .as_ref()
            .map(|o| o.response.trailers.get(name).map(Clone::clone))
            .flatten())
    }

    fn stream_info(&self) -> &dyn StreamInfo {
        self
    }
}
