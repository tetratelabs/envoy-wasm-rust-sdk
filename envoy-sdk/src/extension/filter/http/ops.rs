extern crate std;
use std::prelude::v1::*;

use crate::host;

use proxy_wasm::types::{BufferType, Bytes, MapType};
use proxy_wasm::hostcalls;

pub struct Host;

impl super::RequestHeadersOps for Host {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpRequestHeaders).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn set_request_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestHeaders, headers).map_err(|status| ("proxy_set_header_map_pairs", status))
    }

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpRequestHeaders, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn set_request_header(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestHeaders, &name, value).map_err(|status| match value {
            Some(_) => ("proxy_replace_header_map_value", status),
            None => ("proxy_remove_header_map_value", status),
        })
    }

    fn add_request_header(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpRequestHeaders, &name, value).map_err(|status| ("proxy_add_header_map_value", status))
    }
}

impl super::RequestBodyOps for Host {
    fn get_request_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::HttpRequestBody, start, max_size).map_err(|status| ("proxy_get_buffer_bytes", status))
    }
}

impl super::RequestTrailersOps for Host {
    fn get_request_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpRequestTrailers).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn set_request_trailers(&self, trailers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestTrailers, trailers).map_err(|status| ("proxy_set_header_map_pairs", status))
    }

    fn get_request_trailer(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpRequestTrailers, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn set_request_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestTrailers, &name, value).map_err(|status| match value {
            Some(_) => ("proxy_replace_header_map_value", status),
            None => ("proxy_remove_header_map_value", status),
        })
    }

    fn add_request_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpRequestTrailers, &name, value).map_err(|status| ("proxy_add_header_map_value", status))
    }
}

impl super::ResponseHeadersOps for Host {
    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseHeaders).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn set_response_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseHeaders, headers).map_err(|status| ("proxy_set_header_map_pairs", status))
    }

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseHeaders, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn set_response_header(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseHeaders, &name, value).map_err(|status| match value {
            Some(_) => ("proxy_replace_header_map_value", status),
            None => ("proxy_remove_header_map_value", status),
        })
    }

    fn add_response_header(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpResponseHeaders, &name, value).map_err(|status| ("proxy_add_header_map_value", status))
    }
}

impl super::ResponseBodyOps for Host {
    fn get_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>> {
        hostcalls::get_buffer(BufferType::HttpResponseBody, start, max_size).map_err(|status| ("proxy_get_buffer_bytes", status))
    }
}

impl super::ResponseTrailersOps for Host {
    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseTrailers).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn set_response_trailers(&self, headers: Vec<(&str, &str)>) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseTrailers, headers).map_err(|status| ("proxy_set_header_map_pairs", status))
    }

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseTrailers, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn set_response_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseTrailers, &name, value).map_err(|status| match value {
            Some(_) => ("proxy_replace_header_map_value", status),
            None => ("proxy_remove_header_map_value", status),
        })
    }

    fn add_response_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        hostcalls::add_map_value(MapType::HttpResponseTrailers, &name, value).map_err(|status| ("proxy_add_header_map_value", status))
    }
}

impl super::RequestFlowOps for Host { 
    fn resume_request(&self) -> host::Result<()> {
        hostcalls::resume_http_request().map_err(|status| ("proxy_continue_request", status))
    }

    fn send_response(
        &self,
        status_code: u32,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) -> host::Result<()> {
        hostcalls::send_http_response(status_code, headers, body).map_err(|status| ("proxy_send_local_response", status))
    }

    fn clear_route_cache(&self) -> host::Result<()> {
        hostcalls::clear_http_route_cache().map_err(|status| ("proxy_clear_route_cache", status))
    }
}

impl super::ResponseFlowOps for Host { 
    fn resume_response(&self) -> host::Result<()> {
        hostcalls::resume_http_response().map_err(|status| ("proxy_continue_response", status))
    }
}
