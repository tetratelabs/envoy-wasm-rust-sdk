use proxy_wasm::types::Action;

pub struct FilterContext<'a, F> where F: super::Filter {
    filter: F,
    ops: &'a dyn super::Ops,
}

impl<'a, F> proxy_wasm::traits::HttpContext for FilterContext<'a, F> where F: super::Filter {
    fn on_http_request_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_request_headers(num_headers, self.ops.as_request_headers_ops()).unwrap()
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_request_body(body_size, end_of_stream, self.ops.as_request_body_ops()).unwrap()
    }

    fn on_http_request_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_request_trailers(num_trailers, self.ops.as_request_trailers_ops()).unwrap()
    }

    fn on_http_response_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_response_headers(num_headers, self.ops.as_response_headers_ops()).unwrap()
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_response_body(body_size, end_of_stream, self.ops.as_response_body_ops()).unwrap()
    }

    fn on_http_response_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_response_trailers(num_trailers, self.ops.as_response_trailers_ops()).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_exchange_complete().unwrap()
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
