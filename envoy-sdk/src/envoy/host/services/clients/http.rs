extern crate std;
use std::prelude::v1::*;

use std::time::Duration;

use crate::envoy::host;
use crate::envoy::extension::Result;

use proxy_wasm::types::{BufferType, Bytes, MapType};
use proxy_wasm::hostcalls;

pub trait Client {
    fn send_request(
        &self,
        upstream: &str,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
        trailers: Vec<(&str, &str)>,
        timeout: Duration,
        handler: &dyn ResponseHandler,
    ) -> host::Result<u32>;
}

pub trait ResponseHandler {
    fn on_response(
        &mut self,
        _ops: &dyn ResponseOps,
        _token_id: u32,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait ResponseOps {
    fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_http_call_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;

    fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>>;
}

struct Abi;

impl ResponseOps for Abi {
    fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpCallResponseHeaders)
    }

    fn get_http_call_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, max_size)
    }

    fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpCallResponseTrailers)
    }
}
