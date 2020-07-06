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

use proxy_wasm::types::{Action, PeerType};

use super::{Filter, Ops};
use crate::host::services::clients::http as http_client;

pub struct FilterContext<'a, F>
where
    F: Filter,
{
    filter: F,
    logger_ops: &'a dyn Ops,
    http_client_ops: &'a dyn http_client::ResponseOps,
}

impl<'a, F> proxy_wasm::traits::StreamContext for FilterContext<'a, F>
where
    F: Filter,
{
    fn on_new_connection(&mut self) -> Action {
        self.filter.on_new_connection().unwrap()
    }

    fn on_downstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter
            .on_downstream_data(
                data_size,
                end_of_stream,
                self.logger_ops.as_downstream_data_ops(),
            )
            .unwrap()
    }

    fn on_downstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_downstream_close(peer_type).unwrap()
    }

    fn on_upstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter
            .on_upstream_data(
                data_size,
                end_of_stream,
                self.logger_ops.as_upstream_data_ops(),
            )
            .unwrap()
    }

    fn on_upstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_upstream_close(peer_type).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_connection_complete().unwrap()
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
                self.logger_ops,
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
        logger_ops: &'a dyn Ops,
        http_client_ops: &'a dyn http_client::ResponseOps,
    ) -> FilterContext<'a, F> {
        FilterContext {
            filter,
            logger_ops,
            http_client_ops,
        }
    }

    /// Creates a new network filter context bound to the actual Envoy ABI.
    pub fn with_default_ops(filter: F) -> FilterContext<'a, F> {
        FilterContext::new(filter, &super::ops::Host, &http_client::ops::Host)
    }
}
