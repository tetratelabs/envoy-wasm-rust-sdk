extern crate std;
use std::prelude::v1::*;

use std::time::Duration;

use crate::host;

use proxy_wasm::types::Bytes;

pub type RequestHandle = u32;

pub trait Client {
    fn send_request(
        &self,
        upstream: &str,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
        trailers: Vec<(&str, &str)>,
        timeout: Duration,
    ) -> host::Result<RequestHandle>;
}

pub trait ResponseOps {
    fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_http_call_response_body(
        &self,
        start: usize,
        max_size: usize,
    ) -> host::Result<Option<Bytes>>;

    fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>>;
}

pub mod ops {
    use crate::host;
    use proxy_wasm::hostcalls;
    use proxy_wasm::types::{BufferType, Bytes, MapType};
    use std::time::Duration;

    pub struct Host;

    impl super::Client for Host {
        fn send_request(
            &self,
            upstream: &str,
            headers: Vec<(&str, &str)>,
            body: Option<&[u8]>,
            trailers: Vec<(&str, &str)>,
            timeout: Duration,
        ) -> host::Result<super::RequestHandle> {
            hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
                .map_err(|status| ("proxy_http_call", status))
        }
    }

    impl super::ResponseOps for Host {
        fn get_http_call_response_headers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseHeaders)
                .map_err(|status| ("proxy_get_header_map_pairs", status))
        }

        fn get_http_call_response_body(
            &self,
            start: usize,
            max_size: usize,
        ) -> host::Result<Option<Bytes>> {
            hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, max_size)
                .map_err(|status| ("proxy_get_buffer_bytes", status))
        }

        fn get_http_call_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
            hostcalls::get_map(MapType::HttpCallResponseTrailers)
                .map_err(|status| ("proxy_get_header_map_pairs", status))
        }
    }
}
