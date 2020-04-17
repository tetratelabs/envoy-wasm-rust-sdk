use proxy_wasm::types::Action;

pub struct FilterContext<F, O> where F: super::Filter, O: super::Ops {
    filter: F,
    ops: O,
}

impl<F, O> proxy_wasm::traits::HttpContext for FilterContext<F, O> where F: super::Filter, O: super::Ops {
    fn on_http_request_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_request_headers(num_headers, &self.ops).unwrap()
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_request_body(body_size, end_of_stream, &self.ops).unwrap()
    }

    fn on_http_request_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_request_trailers(num_trailers, &self.ops).unwrap()
    }

    fn on_http_response_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_response_headers(num_headers, &self.ops).unwrap()
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_response_body(body_size, end_of_stream, &self.ops).unwrap()
    }

    fn on_http_response_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_response_trailers(num_trailers, &self.ops).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_exchange_complete().unwrap()
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
