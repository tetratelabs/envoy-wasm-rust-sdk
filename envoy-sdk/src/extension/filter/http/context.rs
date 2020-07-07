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

use proxy_wasm::types::Action;

use super::{Filter, Ops};
use crate::host::services::clients::http as http_client;

pub struct FilterContext<'a, F>
where
    F: Filter,
{
    filter: F,
    filter_ops: &'a dyn Ops,
    http_client_ops: &'a dyn http_client::ResponseOps,
}

impl<'a, F> proxy_wasm::traits::HttpContext for FilterContext<'a, F>
where
    F: Filter,
{
    fn on_http_request_headers(&mut self, num_headers: usize) -> Action {
        self.filter
            .on_request_headers(num_headers, self.filter_ops.as_request_headers_ops())
            .unwrap()
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter
            .on_request_body(
                body_size,
                end_of_stream,
                self.filter_ops.as_request_body_ops(),
            )
            .unwrap()
    }

    fn on_http_request_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter
            .on_request_trailers(num_trailers, self.filter_ops.as_request_trailers_ops())
            .unwrap()
    }

    fn on_http_response_headers(&mut self, num_headers: usize) -> Action {
        self.filter
            .on_response_headers(num_headers, self.filter_ops.as_response_headers_ops())
            .unwrap()
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter
            .on_response_body(
                body_size,
                end_of_stream,
                self.filter_ops.as_response_body_ops(),
            )
            .unwrap()
    }

    fn on_http_response_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter
            .on_response_trailers(num_trailers, self.filter_ops.as_response_trailers_ops())
            .unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_exchange_complete().unwrap()
    }
}

impl<'a, F> proxy_wasm::traits::Context for FilterContext<'a, F>
where
    F: Filter,
{
    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        self.filter
            .on_http_call_response(
                http_client::RequestHandle::from(token_id),
                num_headers,
                body_size,
                num_trailers,
                self.filter_ops,
                self.http_client_ops,
            )
            .unwrap()
    }
}

impl<'a, F> FilterContext<'a, F>
where
    F: Filter,
{
    pub fn new(
        filter: F,
        filter_ops: &'a dyn Ops,
        http_client_ops: &'a dyn http_client::ResponseOps,
    ) -> Self {
        FilterContext {
            filter,
            filter_ops,
            http_client_ops,
        }
    }

    /// Creates a new HTTP filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(filter: F) -> Self {
        FilterContext::new(filter, &super::ops::Host, &http_client::ops::Host)
    }
}
