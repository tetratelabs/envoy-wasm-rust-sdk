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

extern crate std;

use std::fmt;
use std::prelude::v1::*;
use std::time::Duration;

use proxy_wasm::types::Bytes;

use crate::host;

/// Opaque identifier of an ongoing HTTP request.
#[derive(PartialEq, Eq)]
pub struct RequestHandle(u32);

impl From<u32> for RequestHandle {
    fn from(token_id: u32) -> Self {
        RequestHandle(token_id)
    }
}

impl fmt::Display for RequestHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Client {
    fn send_request(
        &self,
        upstream: &str,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
        trailers: Vec<(&str, &str)>,
        timeout: Duration,
    ) -> host::Result<RequestHandle>;
}

pub trait ResponseOps {
    fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_http_call_response_body(
        &self,
        start: usize,
        max_size: usize,
    ) -> host::Result<Option<Bytes>>;

    fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>>;
}

pub mod ops {
    use std::time::Duration;

    use proxy_wasm::hostcalls;
    use proxy_wasm::types::{BufferType, Bytes, MapType};

    use crate::host;

    pub struct Host;

    impl super::Client for Host {
        fn send_request(
            &self,
            upstream: &str,
            headers: Vec<(&str, &str)>,
            body: Option<&[u8]>,
            trailers: Vec<(&str, &str)>,
            timeout: Duration,
        ) -> host::Result<super::RequestHandle> {
            hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
                .map_err(|status| ("proxy_http_call", status))
                .map(super::RequestHandle::from)
        }
    }

    impl super::ResponseOps for Host {
        fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseHeaders)
                .map_err(|status| ("proxy_get_header_map_pairs", status))
        }

        fn get_http_call_response_body(
            &self,
            start: usize,
            max_size: usize,
        ) -> host::Result<Option<Bytes>> {
            hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, max_size)
                .map_err(|status| ("proxy_get_buffer_bytes", status))
        }

        fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseTrailers)
                .map_err(|status| ("proxy_get_header_map_pairs", status))
        }
    }
}
