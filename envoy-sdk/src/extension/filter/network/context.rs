use proxy_wasm::types::{Action, PeerType};

pub struct FilterContext<'a, F> where F: super::Filter {
    filter: F,
    ops: &'a dyn super::Ops,
}

impl<'a, F> proxy_wasm::traits::StreamContext for FilterContext<'a, F> where F: super::Filter {
    fn on_new_connection(&mut self) -> Action {
        self.filter.on_new_connection().unwrap()
    }

    fn on_downstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_downstream_data(data_size, end_of_stream, self.ops.as_downstream_data_ops()).unwrap()
    }

    fn on_downstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_downstream_close(peer_type).unwrap()
    }

    fn on_upstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_upstream_data(data_size, end_of_stream, self.ops.as_upstream_data_ops()).unwrap()
    }

    fn on_upstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_upstream_close(peer_type).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_connection_complete().unwrap()
    }
}

impl<'a, F> proxy_wasm::traits::Context for FilterContext<'a, F> where F: super::Filter {}

impl<'a, F> FilterContext<'a, F> where F: super::Filter {
    pub fn new(filter: F, ops: &'a dyn super::Ops) -> FilterContext<'a, F> {
        FilterContext {
            filter: filter,
            ops: ops,
        }
    }
}
