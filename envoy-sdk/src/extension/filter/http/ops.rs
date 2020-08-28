// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{
    ExchangeCompleteOps, RequestBodyOps, RequestFlowOps, RequestHeadersOps, RequestTrailersOps,
    ResponseBodyOps, ResponseFlowOps, ResponseHeadersOps, ResponseTrailersOps,
};
use crate::abi::proxy_wasm::hostcalls;
use crate::abi::proxy_wasm::types::{BufferType, MapType};
use crate::host::buffer::{Transform, TransformExecutor};
use crate::host::{self, ByteString, HeaderMap};

pub(super) struct Host;

impl RequestHeadersOps for Host {
    fn request_headers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpRequestHeaders)
    }

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpRequestHeaders, name)
    }

    fn set_request_headers(&self, headers: &HeaderMap) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestHeaders, headers)
    }

    fn set_request_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestHeaders, name, Some(value))
    }

    fn remove_request_header(&self, name: &str) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestHeaders, name, None::<&[u8]>)
    }
}

impl RequestBodyOps for Host {
    fn request_data(&self, start: usize, max_size: usize) -> host::Result<ByteString> {
        hostcalls::get_buffer(BufferType::HttpRequestBody, start, max_size)
    }

    fn mutate_request_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, max_size: usize, data: &[u8]| {
            hostcalls::set_buffer(BufferType::HttpRequestBody, start, max_size, data)
        })
    }
}

impl RequestTrailersOps for Host {
    fn request_trailers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpRequestTrailers)
    }

    fn request_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpRequestTrailers, name)
    }

    fn set_request_trailers(&self, trailers: &HeaderMap) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpRequestTrailers, trailers)
    }

    fn set_request_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestTrailers, name, Some(value))
    }

    fn remove_request_trailer(&self, name: &str) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpRequestTrailers, name, None::<&[u8]>)
    }
}

impl ResponseHeadersOps for Host {
    fn response_headers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpResponseHeaders)
    }

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpResponseHeaders, name)
    }

    fn set_response_headers(&self, headers: &HeaderMap) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseHeaders, headers)
    }

    fn set_response_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseHeaders, name, Some(value))
    }

    fn remove_response_header(&self, name: &str) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseHeaders, name, None::<&[u8]>)
    }
}

impl ResponseBodyOps for Host {
    fn response_data(&self, start: usize, max_size: usize) -> host::Result<ByteString> {
        hostcalls::get_buffer(BufferType::HttpResponseBody, start, max_size)
    }

    fn mutate_response_data(&self, change: Transform) -> host::Result<()> {
        change.execute(|start: usize, max_size: usize, data: &[u8]| {
            hostcalls::set_buffer(BufferType::HttpResponseBody, start, max_size, data)
        })
    }
}

impl ResponseTrailersOps for Host {
    fn response_trailers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpResponseTrailers)
    }

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpResponseTrailers, name)
    }

    fn set_response_trailers(&self, trailers: &HeaderMap) -> host::Result<()> {
        hostcalls::set_map(MapType::HttpResponseTrailers, trailers)
    }

    fn set_response_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseTrailers, name, Some(value))
    }

    fn remove_response_trailer(&self, name: &str) -> host::Result<()> {
        hostcalls::set_map_value(MapType::HttpResponseTrailers, name, None::<&[u8]>)
    }
}

impl RequestFlowOps for Host {
    fn resume_request(&self) -> host::Result<()> {
        hostcalls::resume_http_request()
    }

    fn send_response(
        &self,
        status_code: u32,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> host::Result<()> {
        hostcalls::send_http_response(status_code, headers, body)
    }
}

impl ResponseFlowOps for Host {
    fn resume_response(&self) -> host::Result<()> {
        hostcalls::resume_http_response()
    }
}

impl ExchangeCompleteOps for Host {}
