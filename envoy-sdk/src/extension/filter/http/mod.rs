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

/// Return codes for [`on_request_headers`] and [`on_response_headers`] filter
/// invocations.
///
/// `Envoy` bases further filter invocations on the return code of the
/// previous filter.
///
/// [`on_request_headers`]: trait.HttpFilter.html#method.on_request_headers
/// [`on_response_headers`]: trait.HttpFilter.html#method.on_response_headers
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterHeadersStatus {
    /// Continue filter chain iteration.
    Continue = 0,
    /// Do not iterate to any of the remaining filters in the chain.
    ///
    /// To resume filter iteration at a later point, e.g. after the external
    /// authorization request has completed, call [`resume_request`] or
    /// [`resume_response`] respectively.
    ///
    /// [`resume_request`]: trait.RequestFlowOps.html#tymethod.resume_request
    /// [`resume_response`]: trait.ResponseFlowOps.html#tymethod.resume_response
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

/// Return codes for [`on_request_body`] and [`on_response_body`] filter
/// invocations.
///
/// `Envoy` bases further filter invocations on the return code of the
/// previous filter.
///
/// [`on_request_body`]: trait.HttpFilter.html#method.on_request_body
/// [`on_response_body`]: trait.HttpFilter.html#method.on_response_body
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterDataStatus {
    /// Continue filter chain iteration.
    ///
    /// If headers have not yet been sent to the next filter, they
    /// will be sent first. If data has previously been buffered,
    /// the data in this callback will be added to the buffer
    /// before the entirety is sent to the next filter.
    Continue = 0,
    /// Do not iterate to any of the remaining filters in the chain, and buffer body data for later
    /// dispatching.
    ///
    /// To resume filter iteration at a later point, e.g. after enough data has been buffered
    /// to make a decision, call [`resume_request`] or [`resume_response`] respectively.
    ///
    /// This should be called by filters which must parse a larger block of the incoming data before
    /// continuing processing and so can not push back on streaming data via watermarks.
    ///
    /// If buffering the request causes buffered data to exceed the configured buffer limit, a 413 will
    /// be sent to the user. On the response path exceeding buffer limits will result in a 500.
    ///
    /// [`resume_request`]: trait.RequestFlowOps.html#tymethod.resume_request
    /// [`resume_response`]: trait.ResponseFlowOps.html#tymethod.resume_response
    StopIterationAndBuffer = 1,
}

impl FilterDataStatus {
    pub(self) fn as_action(&self) -> Action {
        match self {
            FilterDataStatus::Continue => Action::Continue,
            FilterDataStatus::StopIterationAndBuffer => Action::Pause,
        }
    }
}

/// Return codes for [`on_request_trailers`] and [`on_response_trailers`] filter
/// invocations.
///
/// `Envoy` bases further filter invocations on the return code of the
/// previous filter.
///
/// [`on_request_trailers`]: trait.HttpFilter.html#method.on_request_trailers
/// [`on_response_trailers`]: trait.HttpFilter.html#method.on_response_trailers
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterTrailersStatus {
    /// Continue filter chain iteration.
    Continue = 0,
    /// Do not iterate to any of the remaining filters in the chain.
    ///
    /// To resume filter iteration at a later point, call [`resume_request`] or
    /// [`resume_response`] respectively.
    ///
    /// [`resume_request`]: trait.RequestFlowOps.html#tymethod.resume_request
    /// [`resume_response`]: trait.ResponseFlowOps.html#tymethod.resume_response
    StopIteration = 1,
}

impl FilterTrailersStatus {
    pub(self) fn as_action(&self) -> Action {
        match self {
            FilterTrailersStatus::Continue => Action::Continue,
            FilterTrailersStatus::StopIteration => Action::Pause,
        }
    }
}

/// An interface of the `Envoy` `HTTP Filter` extension.
///
/// `HTTP Filter` operates on a single HTTP stream, i.e. request/response pair.
///
/// When `Envoy` accepts a new connection, a dedicated `HTTP Filter` instance is created for it.
///
/// `HTTP Filter` in `Envoy` is a stateful object.
///
/// **NOTE: This trait MUST NOT panic**. If a filter invocation cannot proceed
/// normally, it should return [`Result::Err(x)`]. In that case, [`Envoy SDK`] will be able to terminate
/// only the affected HTTP request by sending a response with the HTTP Status code
/// `500 (Internal Server Error)`.
/// For comparison, if the extension chooses to panic, this will, at best, affect all ongoing HTTP requests
/// handled by that extension, and, at worst, will crash `Envoy` entirely (as of July 2020).
///
/// [`Result::Err(x)`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
/// [`Envoy SDK`]: https://docs.rs/envoy-sdk
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

    /// Called when HTTP stream is complete.
    ///
    /// This moment happens before `Access Loggers` get called.
    fn on_exchange_complete(&mut self) -> Result<()> {
        Ok(())
    }

    // Http Client callbacks

    /// Called when the async HTTP request made through [`Envoy HTTP Client API`][`HttpClient`] is complete.
    ///
    /// # Arguments
    ///
    /// * `request_id`      - opaque identifier of the request that is now complete.
    /// * `num_headers`     - number of headers in the response.
    /// * `body_size`       - size of the response body.
    /// * `num_trailers`    - number of tarilers in the response.
    /// * `filter_ops`      - a [`trait object`][`Ops`] through which `HTTP Filter` can access data of the HTTP stream it proxies.
    /// * `http_client_ops` - a [`trait object`][`HttpClientResponseOps`] through which `Network Filter` can access
    ///                       data of the response received by [`HttpClient`], including headers, body and trailers.
    ///
    /// [`HttpClient`]: ../../../host/http/client/trait.HttpClient.html
    /// [`HttpClientResponseOps`]: ../../../host/http/client/trait.HttpClientResponseOps.html
    /// [`Ops`]: trait.Ops.html
    fn on_http_call_response(
        &mut self,
        _request_id: HttpClientRequestHandle,
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
    fn request_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn request_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_request_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_request_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait RequestBodyOps: RequestFlowOps {
    fn request_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait RequestTrailersOps: RequestFlowOps {
    fn request_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_trailers(&self, trailers: Vec<(&str, &str)>) -> host::Result<()>;

    fn request_trailer(&self, name: &str) -> host::Result<Option<String>>;

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
    fn response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn response_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_response_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_response_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait ResponseBodyOps: ResponseFlowOps {
    fn response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait ResponseTrailersOps: ResponseFlowOps {
    fn response_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_trailers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn response_trailer(&self, name: &str) -> host::Result<Option<String>>;

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
