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

//! `Envoy` `HTTP Client API`.

use std::time::Duration;

use crate::abi::proxy_wasm_ext::types::Bytes;
pub use crate::abi::proxy_wasm_ext::types::HttpRequestHandle as RequestHandle;
use crate::host;

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

impl dyn Client {
    pub fn default() -> &'static dyn Client {
        &impls::Host
    }
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

impl dyn ResponseOps {
    pub fn default() -> &'static dyn ResponseOps {
        &impls::Host
    }
}

mod impls {
    use std::time::Duration;

    use crate::abi::proxy_wasm_ext::hostcalls;
    use crate::abi::proxy_wasm_ext::types::{BufferType, Bytes, MapType};

    use super::{Client, RequestHandle, ResponseOps};
    use crate::host;

    pub struct Host;

    impl Client for Host {
        fn send_request(
            &self,
            upstream: &str,
            headers: Vec<(&str, &str)>,
            body: Option<&[u8]>,
            trailers: Vec<(&str, &str)>,
            timeout: Duration,
        ) -> host::Result<RequestHandle> {
            hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
        }
    }

    impl ResponseOps for Host {
        fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseHeaders)
        }

        fn get_http_call_response_body(
            &self,
            start: usize,
            max_size: usize,
        ) -> host::Result<Option<Bytes>> {
            hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, max_size)
        }

        fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseTrailers)
        }
    }
}
