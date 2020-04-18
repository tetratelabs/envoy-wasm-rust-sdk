use proxy_wasm::types::{Action, PeerType};

pub struct FilterContext<F, O> where F: super::Filter, O: super::Ops {
    filter: F,
    ops: O,
}

impl<F, O> proxy_wasm::traits::StreamContext for FilterContext<F, O> where F: super::Filter, O: super::Ops {
    fn on_new_connection(&mut self) -> Action {
        self.filter.on_new_connection().unwrap()
    }

    fn on_downstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_downstream_data(data_size, end_of_stream, &self.ops).unwrap()
    }

    fn on_downstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_downstream_close(peer_type).unwrap()
    }

    fn on_upstream_data(&mut self, data_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_upstream_data(data_size, end_of_stream, &self.ops).unwrap()
    }

    fn on_upstream_close(&mut self, peer_type: PeerType) {
        self.filter.on_upstream_close(peer_type).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_connection_complete().unwrap()
    }
}

impl<F, O> proxy_wasm::traits::Context for FilterContext<F, O> where F: super::Filter, O: super::Ops {}

impl<F, O>  FilterContext<F, O> where F: super::Filter, O: super::Ops {
    pub fn new(filter: F, ops: O) -> FilterContext<F, O> {
        FilterContext {
            filter: filter,
            ops: ops,
        }
    }
}
