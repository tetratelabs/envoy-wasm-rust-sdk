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

//! `Envoy` `HTTP Filter API`.

use crate::abi::proxy_wasm::types::{Action, Bytes};
use crate::extension::Result;
use crate::host;
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};

pub(crate) use self::context::{HttpFilterContext, VoidHttpFilterContext};

mod context;
mod ops;

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterHeadersStatus {
    Continue = 0,
    StopIteration = 1,
}

impl FilterHeadersStatus {
    pub(self) fn as_action(&self) -> Action {
        match self {
            FilterHeadersStatus::Continue => Action::Continue,
            FilterHeadersStatus::StopIteration => Action::Pause,
        }
    }
}

pub type FilterDataStatus = FilterHeadersStatus;
pub type FilterTrailersStatus = FilterHeadersStatus;

pub trait HttpFilter {
    fn on_request_headers(
        &mut self,
        _num_headers: usize,
        _ops: &dyn RequestHeadersOps,
    ) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    fn on_request_body(
        &mut self,
        _body_size: usize,
        _end_of_stream: bool,
        _ops: &dyn RequestBodyOps,
    ) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    fn on_request_trailers(
        &mut self,
        _num_trailers: usize,
        _ops: &dyn RequestTrailersOps,
    ) -> Result<FilterTrailersStatus> {
        Ok(FilterTrailersStatus::Continue)
    }

    fn on_response_headers(
        &mut self,
        _num_headers: usize,
        _ops: &dyn ResponseHeadersOps,
    ) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    fn on_response_body(
        &mut self,
        _body_size: usize,
        _end_of_stream: bool,
        _ops: &dyn ResponseBodyOps,
    ) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    fn on_response_trailers(
        &mut self,
        _num_trailers: usize,
        _ops: &dyn ResponseTrailersOps,
    ) -> Result<FilterTrailersStatus> {
        Ok(FilterTrailersStatus::Continue)
    }

    fn on_exchange_complete(&mut self) -> Result<()> {
        Ok(())
    }

    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        _request: HttpClientRequestHandle,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        _filter_ops: &dyn Ops,
        _http_client_ops: &dyn HttpClientResponseOps,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait RequestHeadersOps: RequestFlowOps {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_request_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_request_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait RequestBodyOps: RequestFlowOps {
    fn get_request_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait RequestTrailersOps: RequestFlowOps {
    fn get_request_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_trailers(&self, trailers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_request_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn set_request_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_request_trailer(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait RequestFlowOps {
    fn resume_request(&self) -> host::Result<()>;

    fn clear_route_cache(&self) -> host::Result<()>;

    fn send_response(
        &self,
        status_code: u32,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) -> host::Result<()>;
}

pub trait ResponseHeadersOps: ResponseFlowOps {
    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_response_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_response_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait ResponseBodyOps: ResponseFlowOps {
    fn get_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait ResponseTrailersOps: ResponseFlowOps {
    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_trailers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn set_response_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_response_trailer(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait ResponseFlowOps {
    fn resume_response(&self) -> host::Result<()>;
}

pub trait Ops:
    RequestHeadersOps
    + RequestBodyOps
    + RequestTrailersOps
    + ResponseHeadersOps
    + ResponseBodyOps
    + ResponseTrailersOps
{
    fn as_request_headers_ops(&self) -> &dyn RequestHeadersOps;

    fn as_request_body_ops(&self) -> &dyn RequestBodyOps;

    fn as_request_trailers_ops(&self) -> &dyn RequestTrailersOps;

    fn as_response_headers_ops(&self) -> &dyn ResponseHeadersOps;

    fn as_response_body_ops(&self) -> &dyn ResponseBodyOps;

    fn as_response_trailers_ops(&self) -> &dyn ResponseTrailersOps;
}

impl<T> Ops for T
where
    T: RequestHeadersOps
        + RequestBodyOps
        + RequestTrailersOps
        + ResponseHeadersOps
        + ResponseBodyOps
        + ResponseTrailersOps,
{
    fn as_request_headers_ops(&self) -> &dyn RequestHeadersOps {
        self
    }

    fn as_request_body_ops(&self) -> &dyn RequestBodyOps {
        self
    }

    fn as_request_trailers_ops(&self) -> &dyn RequestTrailersOps {
        self
    }

    fn as_response_headers_ops(&self) -> &dyn ResponseHeadersOps {
        self
    }

    fn as_response_body_ops(&self) -> &dyn ResponseBodyOps {
        self
    }

    fn as_response_trailers_ops(&self) -> &dyn ResponseTrailersOps {
        self
    }
}

impl dyn Ops {
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
