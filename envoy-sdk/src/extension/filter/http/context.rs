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

use crate::abi::proxy_wasm::traits::{Context, HttpContext};
use crate::abi::proxy_wasm::types::Action;

use super::{FilterDataStatus, FilterHeadersStatus, FilterTrailersStatus, HttpFilter, Ops};
use crate::extension::error::ErrorSink;
use crate::extension::Error;
use crate::host::{HttpClientRequestHandle, HttpClientResponseOps};

pub(crate) struct HttpFilterContext<'a, F>
where
    F: HttpFilter,
{
    filter: F,
    filter_ops: &'a dyn Ops,
    http_client_ops: &'a dyn HttpClientResponseOps,
    error_sink: &'a dyn ErrorSink,
}

impl<'a, F> HttpContext for HttpFilterContext<'a, F>
where
    F: HttpFilter,
{
    fn on_http_request_headers(&mut self, num_headers: usize) -> Action {
        match self
            .filter
            .on_request_headers(num_headers, self.filter_ops.as_request_headers_ops())
        {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP request headers", &err);
                self.handle_error(err);
                FilterHeadersStatus::StopIteration.as_action()
            }
        }
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        match self.filter.on_request_body(
            body_size,
            end_of_stream,
            self.filter_ops.as_request_body_ops(),
        ) {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP request body", &err);
                self.handle_error(err);
                FilterDataStatus::StopIteration.as_action()
            }
        }
    }

    fn on_http_request_trailers(&mut self, num_trailers: usize) -> Action {
        match self
            .filter
            .on_request_trailers(num_trailers, self.filter_ops.as_request_trailers_ops())
        {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP request trailers", &err);
                self.handle_error(err);
                FilterTrailersStatus::StopIteration.as_action()
            }
        }
    }

    fn on_http_response_headers(&mut self, num_headers: usize) -> Action {
        match self
            .filter
            .on_response_headers(num_headers, self.filter_ops.as_response_headers_ops())
        {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP response headers", &err);
                self.handle_error(err);
                FilterHeadersStatus::StopIteration.as_action()
            }
        }
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        match self.filter.on_response_body(
            body_size,
            end_of_stream,
            self.filter_ops.as_response_body_ops(),
        ) {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP response body", &err);
                self.handle_error(err);
                FilterDataStatus::StopIteration.as_action()
            }
        }
    }

    fn on_http_response_trailers(&mut self, num_trailers: usize) -> Action {
        match self
            .filter
            .on_response_trailers(num_trailers, self.filter_ops.as_response_trailers_ops())
        {
            Ok(status) => status.as_action(),
            Err(err) => {
                self.error_sink
                    .observe("failed to handle HTTP response trailers", &err);
                self.handle_error(err);
                FilterTrailersStatus::StopIteration.as_action()
            }
        }
    }

    fn on_log(&mut self) {
        if let Err(err) = self.filter.on_exchange_complete() {
            self.error_sink
                .observe("failed to handle completion of an HTTP stream", &err);
            // HTTP stream is already being terminated, so there is no need to do it explicitly
        }
    }
}

impl<'a, F> Context for HttpFilterContext<'a, F>
where
    F: HttpFilter,
{
    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        if let Err(err) = self.filter.on_http_call_response(
            HttpClientRequestHandle::from(token_id),
            num_headers,
            body_size,
            num_trailers,
            self.filter_ops,
            self.http_client_ops,
        ) {
            self.error_sink.observe(
                "failed to process a response to an HTTP request made by the extension",
                &err,
            );
            self.handle_error(err);
        }
    }
}

impl<'a, F> HttpFilterContext<'a, F>
where
    F: HttpFilter,
{
    pub fn new(
        filter: F,
        filter_ops: &'a dyn Ops,
        http_client_ops: &'a dyn HttpClientResponseOps,
        error_sink: &'a dyn ErrorSink,
    ) -> Self {
        HttpFilterContext {
            filter,
            filter_ops,
            http_client_ops,
            error_sink,
        }
    }

    /// Creates a new HTTP filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(filter: F) -> Self {
        Self::new(
            filter,
            Ops::default(),
            HttpClientResponseOps::default(),
            ErrorSink::default(),
        )
    }

    fn handle_error(&self, _err: Error) {
        if let Err(err) = self.filter_ops.send_response(500, vec![], None) {
            self.error_sink.observe(
                "failed to terminate processing of the HTTP request: failed to send a direct reply",
                &err,
            );
        }
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
pub(crate) struct VoidHttpFilterContext<'a> {
    err: Error,
    filter_ops: &'a dyn Ops,
    error_sink: &'a dyn ErrorSink,
}

impl<'a> VoidHttpFilterContext<'a> {
    pub fn new(err: Error, filter_ops: &'a dyn Ops, error_sink: &'a dyn ErrorSink) -> Self {
        VoidHttpFilterContext {
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

impl<'a> HttpContext for VoidHttpFilterContext<'a> {
    fn on_http_request_headers(&mut self, _num_headers: usize) -> Action {
        self.error_sink
            .observe("failed to create Proxy Wasm Http Context", &self.err);
        if let Err(err) = self.filter_ops.send_response(500, vec![], None) {
            self.error_sink.observe(
                "failed to terminate processing of the HTTP request: failed to send a direct reply",
                &err,
            );
        }
        FilterHeadersStatus::StopIteration.as_action()
    }
}

impl<'a> Context for VoidHttpFilterContext<'a> {}
