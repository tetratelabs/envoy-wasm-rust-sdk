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

use super::{ConfigureOps, ContextOps, DrainOps, LogOps, StreamInfo};
use crate::abi::proxy_wasm::hostcalls;
use crate::abi::proxy_wasm::types::MapType;
use crate::host::{self, ByteString, HeaderMap};

pub(super) struct Host;

impl ContextOps for Host {
    fn configuration(&self) -> host::Result<ByteString> {
        hostcalls::get_configuration()
    }
}

impl ConfigureOps for Host {}

impl LogOps for Host {
    fn request_headers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpRequestHeaders)
    }

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpRequestHeaders, name)
    }

    fn response_headers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpResponseHeaders)
    }

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpResponseHeaders, name)
    }

    fn response_trailers(&self) -> host::Result<HeaderMap> {
        hostcalls::get_map(MapType::HttpResponseTrailers)
    }

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>> {
        hostcalls::get_map_value(MapType::HttpResponseTrailers, &name)
    }

    fn stream_info(&self) -> &dyn StreamInfo {
        StreamInfo::default()
    }
}

impl DrainOps for Host {
    fn done(&self) -> host::Result<()> {
        hostcalls::done()
    }
}
