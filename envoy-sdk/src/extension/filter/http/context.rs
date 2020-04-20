use proxy_wasm::types::Action;

use crate::host::services::clients;

pub struct FilterContext<'a, F> where F: super::Filter {
    filter: F,
    filter_ops: &'a dyn super::Ops,
    http_client_ops: &'a dyn clients::http::ResponseOps,
}

impl<'a, F> proxy_wasm::traits::HttpContext for FilterContext<'a, F> where F: super::Filter {
    fn on_http_request_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_request_headers(num_headers, self.filter_ops.as_request_headers_ops()).unwrap()
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_request_body(body_size, end_of_stream, self.filter_ops.as_request_body_ops()).unwrap()
    }

    fn on_http_request_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_request_trailers(num_trailers, self.filter_ops.as_request_trailers_ops()).unwrap()
    }

    fn on_http_response_headers(&mut self, num_headers: usize) -> Action {
        self.filter.on_response_headers(num_headers, self.filter_ops.as_response_headers_ops()).unwrap()
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        self.filter.on_response_body(body_size, end_of_stream, self.filter_ops.as_response_body_ops()).unwrap()
    }

    fn on_http_response_trailers(&mut self, num_trailers: usize) -> Action {
        self.filter.on_response_trailers(num_trailers, self.filter_ops.as_response_trailers_ops()).unwrap()
    }

    fn on_log(&mut self) {
        self.filter.on_exchange_complete().unwrap()
    }
}

impl<'a, F> proxy_wasm::traits::Context for FilterContext<'a, F> where F: super::Filter {

    // Http Client callbacks

    fn on_http_call_response(&mut self, token_id: u32, num_headers: usize, body_size: usize, num_trailers: usize) {
        self.filter.on_http_call_response(token_id, num_headers, body_size, num_trailers, self.filter_ops, self.http_client_ops).unwrap()
    }
}

impl<'a, F> FilterContext<'a, F> where F: super::Filter {
    pub fn new(filter: F, filter_ops: &'a dyn super::Ops, http_client_ops: &'a dyn clients::http::ResponseOps) -> FilterContext<'a, F> {
        FilterContext {
            filter: filter,
            filter_ops: filter_ops,
            http_client_ops: http_client_ops,
        }
    }
}
