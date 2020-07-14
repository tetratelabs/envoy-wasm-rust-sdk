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

use super::{Filter, Ops};
use crate::abi::proxy_wasm_ext::traits::{Context, StreamContext};
use crate::abi::proxy_wasm_ext::types::{Action, PeerType};
use crate::extension::error::ErrorSink;
use crate::extension::Error;
use crate::host::http::client as http_client;

pub(crate) struct FilterContext<'a, F>
where
    F: Filter,
{
    filter: F,
    logger_ops: &'a dyn Ops,
    http_client_ops: &'a dyn http_client::ResponseOps,
    error_sink: &'a dyn ErrorSink,
}

impl<'a, F> StreamContext for FilterContext<'a, F>
where
    F: Filter,
{
    fn on_new_connection(&mut self) -> Action {
        match self.filter.on_new_connection() {
            Ok(action) => action,
            Err(err) => {
                self.error_sink
                    .observe("failed to handle connection opening", &err);
                self.handle_error(err);
                Action::Pause
            }
        }
    }

    fn on_downstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        match self.filter.on_downstream_data(
            data_size,
            end_of_stream,
            self.logger_ops.as_downstream_data_ops(),
        ) {
            Ok(action) => action,
            Err(err) => {
                self.error_sink
                    .observe("failed to handle data from the downstream", &err);
                self.handle_error(err);
                Action::Pause
            }
        }
    }

    fn on_downstream_close(&mut self, peer_type: PeerType) {
        if let Err(err) = self.filter.on_downstream_close(peer_type) {
            self.error_sink
                .observe("failed to handle connection close by the downstream", &err);
            // TODO(yskopets): do we still need to do anything to terminate the connection?
            self.handle_error(err);
        }
    }

    fn on_upstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        match self.filter.on_upstream_data(
            data_size,
            end_of_stream,
            self.logger_ops.as_upstream_data_ops(),
        ) {
            Ok(action) => action,
            Err(err) => {
                self.error_sink
                    .observe("failed to handle data from the upstream", &err);
                self.handle_error(err);
                Action::Pause
            }
        }
    }

    fn on_upstream_close(&mut self, peer_type: PeerType) {
        if let Err(err) = self.filter.on_upstream_close(peer_type) {
            self.error_sink
                .observe("failed to handle connection close by the upstream", &err);
            // TODO(yskopets): do we still need to do anything to terminate the connection?
            self.handle_error(err);
        }
    }

    fn on_log(&mut self) {
        if let Err(err) = self.filter.on_connection_complete() {
            self.error_sink
                .observe("failed to handle completion of a connection", &err);
            // connection is already being terminated, so there is no need to do it explicitly
        }
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
        if let Err(err) = self.filter.on_http_call_response(
            http_client::RequestHandle::from(token_id),
            num_headers,
            body_size,
            num_trailers,
            self.logger_ops,
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

impl<'a, F> FilterContext<'a, F>
where
    F: Filter,
{
    pub fn new(
        filter: F,
        logger_ops: &'a dyn Ops,
        http_client_ops: &'a dyn http_client::ResponseOps,
        error_sink: &'a dyn ErrorSink,
    ) -> Self {
        FilterContext {
            filter,
            logger_ops,
            http_client_ops,
            error_sink,
        }
    }

    /// Creates a new network filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(filter: F) -> Self {
        Self::new(
            filter,
            Ops::default(),
            http_client::ResponseOps::default(),
            ErrorSink::default(),
        )
    }

    fn handle_error(&self, _err: Error) {
        // TODO(yskopets): Proxy Wasm should provide ABI for closing the downstream connection
        // https://github.com/tetratelabs/envoy-wasm-rust-sdk/issues/29
    }
}

/// Fake `Proxy Wasm` [`StreamContext`] that is used to postpone error handling
/// until a proper moment in the connection lifecycle.
///
/// E.g., if an error occurres inside [`proxy_on_context_create`] callback
/// where a new Network Filter instance is supposed to be created,
/// we cannot terminate the L4 connection right away - `Envoy` doesn't expect it
/// at this point.
///
/// Instead, we have to memorize the error and wait until [`proxy_on_new_connection`]
/// callback when it will be safe to use [`not yet supported ABI`] to stop further processing.
///
/// [`StreamContext`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/traits/trait.StreamContext.html
/// [`proxy_on_context_create`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_context_create
/// [`proxy_on_new_connection`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_new_connection
/// [`not yet supported ABI`]: https://github.com/tetratelabs/envoy-wasm-rust-sdk/issues/29
pub(crate) struct VoidFilterContext<'a> {
    err: Error,
    _filter_ops: &'a dyn Ops,
    error_sink: &'a dyn ErrorSink,
}

impl<'a> VoidFilterContext<'a> {
    pub fn new(err: Error, _filter_ops: &'a dyn Ops, error_sink: &'a dyn ErrorSink) -> Self {
        VoidFilterContext {
            err,
            _filter_ops,
            error_sink,
        }
    }

    /// Creates a new HTTP filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(err: Error) -> Self {
        Self::new(err, Ops::default(), ErrorSink::default())
    }
}

impl<'a> StreamContext for VoidFilterContext<'a> {
    fn on_new_connection(&mut self) -> Action {
        self.error_sink
            .observe("failed to create Proxy Wasm Stream Context", &self.err);
        // TODO(yskopets): Proxy Wasm should provide ABI for closing the downstream connection
        // https://github.com/tetratelabs/envoy-wasm-rust-sdk/issues/29
        Action::Pause
    }
}

impl<'a> Context for VoidFilterContext<'a> {}
