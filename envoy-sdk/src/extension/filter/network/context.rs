use proxy_wasm::types::{Action, PeerType};

use crate::host::services::clients;

pub struct FilterContext<'a, F>
where
    F: super::Filter,
{
    filter: F,
    logger_ops: &'a dyn super::Ops,
    http_client_ops: &'a dyn clients::http::ResponseOps,
}

impl<'a, F> proxy_wasm::traits::StreamContext for FilterContext<'a, F>
where
    F: super::Filter,
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
    F: super::Filter,
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
                token_id,
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
    F: super::Filter,
{
    pub fn new(
        filter: F,
        logger_ops: &'a dyn super::Ops,
        http_client_ops: &'a dyn clients::http::ResponseOps,
    ) -> FilterContext<'a, F> {
        FilterContext {
            filter,
            logger_ops,
            http_client_ops,
        }
    }
}
