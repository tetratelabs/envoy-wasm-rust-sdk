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

use crate::abi::proxy_wasm_ext::traits::{Context, HttpContext};
use crate::abi::proxy_wasm_ext::types::Action;

use super::{Filter, Ops};
use crate::extension::error::ErrorSink;
use crate::extension::Error;
use crate::host::http::client as http_client;

pub(crate) struct FilterContext<'a, F>
where
    F: Filter,
{
    filter: F,
    filter_ops: &'a dyn Ops,
    http_client_ops: &'a dyn http_client::ResponseOps,
}

impl<'a, F> HttpContext for FilterContext<'a, F>
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

impl<'a, F> Context for FilterContext<'a, F>
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
        Self::new(filter, Ops::default(), http_client::ResponseOps::default())
    }
}

/// Fake `Proxy Wasm` [`HttpContext`] that is used to postpone error handling
/// until a proper moment in the request lifecycle.
///
/// E.g., if an error occurres inside [`proxy_on_context_create`] callback
/// where a new HTTP Filter instance is supposed to be created,
/// we cannot terminate the HTTP request right away - `Envoy` doesn't expect it
/// at this point.
///
/// Instead, we have to memorize the error and wait until [`proxy_on_http_request_headers`]
/// callback when it will be safe to use [`proxy_send_http_response`] to stop further processing.
///
/// [`HttpContext`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/traits/trait.HttpContext.html
/// [`proxy_on_context_create`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_context_create
/// [`proxy_on_http_request_headers`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_http_request_headers
/// [`proxy_send_http_response`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_send_http_response
pub(crate) struct VoidFilterContext<'a> {
    err: Error,
    filter_ops: &'a dyn Ops,
    error_sink: &'a dyn ErrorSink,
}

impl<'a> VoidFilterContext<'a> {
    pub fn new(err: Error, filter_ops: &'a dyn Ops, error_sink: &'a dyn ErrorSink) -> Self {
        VoidFilterContext {
            err,
            filter_ops,
            error_sink,
        }
    }

    /// Creates a new HTTP filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(err: Error) -> Self {
        Self::new(err, Ops::default(), ErrorSink::default())
    }
}

impl<'a> HttpContext for VoidFilterContext<'a> {
    fn on_http_request_headers(&mut self, _num_headers: usize) -> Action {
        self.error_sink
            .observe("failed to create Proxy Wasm Http Context", &self.err);
        if let Err(err) = self.filter_ops.send_response(500, vec![], None) {
            self.error_sink.observe(
                "failed to terminate processing of the HTTP request: failed to send a direct reply",
                &err,
            );
        }
        Action::Pause
    }
}

impl<'a> Context for VoidFilterContext<'a> {}
