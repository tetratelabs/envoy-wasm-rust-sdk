extern crate std;
use std::prelude::v1::*;

use crate::host;

use proxy_wasm::hostcalls;
use proxy_wasm::types::{Bytes, MapType};
pub struct Host;

impl super::ConfigureOps for Host {
    fn get_configuration(&self) -> host::Result<Option<Bytes>> {
        hostcalls::get_configuration().map_err(|status| ("proxy_get_configuration", status))
    }
}

impl super::LogOps for Host {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpRequestHeaders).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpRequestHeaders, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseHeaders).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseHeaders, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>> {
        hostcalls::get_map(MapType::HttpResponseTrailers).map_err(|status| ("proxy_get_header_map_pairs", status))
    }

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>> {
        hostcalls::get_map_value(MapType::HttpResponseTrailers, &name).map_err(|status| ("proxy_get_header_map_value", status))
    }

    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>> {
        hostcalls::get_property(path).map_err(|status| ("proxy_get_property", status))
    }
}
