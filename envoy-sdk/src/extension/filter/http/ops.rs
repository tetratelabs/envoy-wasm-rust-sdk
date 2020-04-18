extern crate std;
use std::prelude::v1::*;

use crate::host;

use proxy_wasm::types::{BufferType, Bytes, MapType};
use proxy_wasm::hostcalls;

pub struct Host;

impl super::RequestHeadersOps for Host {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpRequestHeaders)
    }

    fn set_request_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestHeaders, headers)
    }

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpRequestHeaders, &name)
    }

    fn set_request_header(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestHeaders, &name, value)
    }

    fn add_request_header(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpRequestHeaders, &name, value)
    }
}

impl super::RequestBodyOps for Host {
    fn get_request_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::HttpRequestBody, start, max_size)
    }
}

impl super::RequestTrailersOps for Host {
    fn get_request_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpRequestTrailers)
    }

    fn set_request_trailers(&self, trailers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestTrailers, trailers)
    }

    fn get_request_trailer(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpRequestTrailers, &name)
    }

    fn set_request_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestTrailers, &name, value)
    }

    fn add_request_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpRequestTrailers, &name, value)
    }
}

impl super::ResponseHeadersOps for Host {
    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseHeaders)
    }

    fn set_response_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseHeaders, headers)
    }

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseHeaders, &name)
    }

    fn set_response_header(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseHeaders, &name, value)
    }

    fn add_response_header(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpResponseHeaders, &name, value)
    }
}

impl super::ResponseBodyOps for Host {
    fn get_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::HttpResponseBody, start, max_size)
    }
}

impl super::ResponseTrailersOps for Host {
    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseTrailers)
    }

    fn set_response_trailers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseTrailers, headers)
    }

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseTrailers, &name)
    }

    fn set_response_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseTrailers, &name, value)
    }

    fn add_response_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpResponseTrailers, &name, value)
    }
}

impl super::RequestFlowOps for Host { 
    fn resume_request(&self) -> host::Result<()> {
        hostcalls::resume_http_request()
    }

    fn send_response(
        &self,
        status_code: u32,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) -> host::Result<()> {
        hostcalls::send_http_response(status_code, headers, body)
    }

    fn clear_route_cache(&self) -> host::Result<()> {
        hostcalls::clear_http_route_cache()
    }
}

impl super::ResponseFlowOps for Host { 
    fn resume_response(&self) -> host::Result<()> {
        hostcalls::resume_http_response()
    }
}
