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

//! `Envoy` `HTTP Filter` extension.
//!
//! Creating a new `HTTP Filter` extension using `Envoy SDK` consists of the following steps:
//!
//! 1. Implement [`HttpFilter`] trait to define core logic of your extension
//! 2. Implement [`ExtensionFactory`] trait to create new instances of your extension
//! 3. [`Register`] your extension on WebAssembly module start up
//!
//! # Examples
//!
//! #### Basic [`HttpFilter`]:
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::extension::HttpFilter;
//!
//! /// My very own `HttpFilter`.
//! struct MyHttpFilter;
//!
//! impl HttpFilter for MyHttpFilter {}
//! ```
//!
//! #### [`ExtensionFactory`] for `MyHttpFilter` instances:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::HttpFilter;
//! #
//! # /// My very own `HttpFilter`.
//! # struct MyHttpFilter;
//! #
//! # impl HttpFilter for MyHttpFilter {}
//! #
//! use envoy::extension::{ExtensionFactory, InstanceId, Result};
//!
//! /// `ExtensionFactory` for `MyHttpFilter`.
//! struct MyHttpFilterFactory;
//!
//! impl ExtensionFactory for MyHttpFilterFactory {
//!     type Extension = MyHttpFilter;
//!
//!     fn name() -> &'static str { "my_http_filter" }
//!
//!     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
//!         Ok(MyHttpFilter)
//!     }
//! }
//! ```
//!
//! #### Registration of `MyHttpFilter` on start up:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::HttpFilter;
//! #
//! # /// My very own `HttpFilter`.
//! # struct MyHttpFilter;
//! # impl HttpFilter for MyHttpFilter {}
//! #
//! # use envoy::extension::{ExtensionFactory, InstanceId, self};
//! #
//! # /// `ExtensionFactory` for `MyHttpFilter`.
//! # struct MyHttpFilterFactory;
//! # impl ExtensionFactory for MyHttpFilterFactory {
//! #     type Extension = MyHttpFilter;
//! #
//! #     fn name() -> &'static str { "my_http_filter" }
//! #
//! #     fn new_extension(&mut self, _instance_id: InstanceId) -> extension::Result<Self::Extension> {
//! #         Ok(MyHttpFilter)
//! #     }
//! # }
//! #
//! use envoy::extension::{entrypoint, Module, Result};
//!
//! entrypoint! { initialize } // put initialization logic into a function to make it unit testable
//!
//! fn initialize() -> Result<Module> {
//!     Module::new()
//!         .add_http_filter(|_instance_id| Ok(MyHttpFilterFactory))
//! }
//! ```
//!
//! [`HttpFilter`]: trait.HttpFilter.html
//! [`ExtensionFactory`]: ../../factory/trait.ExtensionFactory.html
//! [`Register`]: ../../../macro.entrypoint.html

use crate::abi::proxy_wasm::types::Action;
use crate::extension::Result;
use crate::host::buffer::Transform;
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};
use crate::host::{self, ByteString, HeaderMap};

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
/// A dedicated `HTTP Filter` instance is created for every `HTTP/1.1` request
/// or `HTTP/2` stream handled by `Envoy`.
///
/// Consequently, state of a single HTTP stream can be stored inside `HTTP Filter` itself.
///
/// # Examples
///
/// #### Basic `HttpFilter`:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::extension::{HttpFilter, Result};
/// use envoy::extension::filter::http::{FilterHeadersStatus, RequestHeadersOps};
/// use envoy::host::log;
///
/// /// My very own `HttpFilter`.
/// struct MyHttpFilter;
///
/// impl HttpFilter for MyHttpFilter {
///     fn on_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool, ops: &dyn RequestHeadersOps) -> Result<FilterHeadersStatus> {
///         let user_agent = ops.request_header("user-agent")?.unwrap_or_else(|| "<unknown>".into());
///         log::info!("user-agent: {}", user_agent);
///         Ok(FilterHeadersStatus::Continue)
///     }
/// }
/// ```
///
/// # NOTE
///
/// **This trait MUST NOT panic!**
///
/// If a filter invocation cannot proceed normally, it should return [`Result::Err(x)`].
/// In that case, `Envoy SDK` will be able to terminate
/// only the affected HTTP request by sending a response with the HTTP Status code
/// `500 (Internal Server Error)`.
///
/// For comparison, if the extension chooses to panic, this will, at best, affect all ongoing HTTP requests
/// handled by that extension, and, at worst, will crash `Envoy` entirely (as of July 2020).
///
/// [`Result::Err(x)`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
pub trait HttpFilter {
    /// Called with decoded request headers.
    ///
    /// # Arguments
    ///
    /// * `num_headers` - number of headers in the request.
    /// * `ops`         - a [`trait object`][`RequestHeadersOps`] through which `HTTP Filter` can
    ///                   manipulate request headers.
    ///
    /// # Return value
    ///
    /// [`FilterHeadersStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterHeadersStatus`]: enum.FilterHeadersStatus.html
    /// [`RequestHeadersOps`]: trait.RequestHeadersOps.html
    ///
    /// # Examples
    ///
    /// #### Basic usage to sniff request headers:
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// # use envoy::extension::{HttpFilter, Result};
    /// # use envoy::extension::filter::http::{FilterHeadersStatus, RequestHeadersOps};
    /// # use envoy::host::log;
    /// #
    /// # /// My very own `HttpFilter`.
    /// # struct MyHttpFilter;
    /// #
    /// # impl HttpFilter for MyHttpFilter {
    ///   fn on_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool, ops: &dyn RequestHeadersOps) -> Result<FilterHeadersStatus> {
    ///       let user_agent = ops.request_header("user-agent")?.unwrap_or_else(|| "<unknown>".into());
    ///       log::info!("user-agent: {}", user_agent);
    ///       Ok(FilterHeadersStatus::Continue)
    ///   }
    /// # }
    /// ```
    fn on_request_headers(
        &mut self,
        _num_headers: usize,
        _end_of_stream: bool,
        _ops: &dyn RequestHeadersOps,
    ) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    /// Called with a decoded request data frame.
    ///
    /// # Arguments
    ///
    /// * `data_size`     - size of data accumulated in the read buffer.
    /// * `end_of_stream` - supplies whether this is the last data frame.
    /// * `ops`           - a [`trait object`][`RequestBodyOps`] through which `HTTP Filter` can
    ///                     manipulate request body.
    ///
    /// # Return value
    ///
    /// [`FilterDataStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterDataStatus`]: enum.FilterDataStatus.html
    /// [`RequestBodyOps`]: trait.RequestBodyOps.html
    ///
    /// # Examples
    ///
    /// #### Basic usage to sniff request body:
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// # use envoy::extension::{HttpFilter, Result};
    /// # use envoy::extension::filter::http::{FilterDataStatus, RequestBodyOps};
    /// # use envoy::host::log;
    /// #
    /// # /// My very own `HttpFilter`.
    /// # struct MyHttpFilter;
    /// #
    /// # impl HttpFilter for MyHttpFilter {
    ///   fn on_request_body(&mut self, _data_size: usize, _end_of_stream: bool, ops: &dyn RequestBodyOps) -> Result<FilterDataStatus> {
    ///       let head = ops.request_data(0, 10)?;
    ///       log::info!("body chunk starts with: {:?}", head);
    ///       Ok(FilterDataStatus::Continue)
    ///   }
    /// # }
    /// ```
    fn on_request_body(
        &mut self,
        _data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn RequestBodyOps,
    ) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    /// Called with decoded trailers, implicitly ending the stream.
    ///
    ///
    /// # Arguments
    ///
    /// * `num_trailers` - number of trailers in the request.
    /// * `ops`          - a [`trait object`][`RequestTrailersOps`] through which `HTTP Filter` can
    ///                    manipulate request trailers.
    ///
    /// # Return value
    ///
    /// [`FilterTrailersStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterTrailersStatus`]: enum.FilterTrailersStatus.html
    /// [`RequestTrailersOps`]: trait.RequestTrailersOps.html
    ///
    /// # Examples
    ///
    /// #### Basic usage to sniff request trailers:
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// # use envoy::extension::{HttpFilter, Result};
    /// # use envoy::extension::filter::http::{FilterTrailersStatus, RequestTrailersOps};
    /// # use envoy::host::log;
    /// #
    /// # /// My very own `HttpFilter`.
    /// # struct MyHttpFilter;
    /// #
    /// # impl HttpFilter for MyHttpFilter {
    ///   fn on_request_trailers(&mut self, _num_headers: usize, ops: &dyn RequestTrailersOps) -> Result<FilterTrailersStatus> {
    ///       let grpc_message = ops.request_trailer("grpc-message")?.unwrap_or_else(|| "<unknown>".into());
    ///       log::info!("grpc-message: {}", grpc_message);
    ///       Ok(FilterTrailersStatus::Continue)
    ///   }
    /// # }
    /// ```
    fn on_request_trailers(
        &mut self,
        _num_trailers: usize,
        _ops: &dyn RequestTrailersOps,
    ) -> Result<FilterTrailersStatus> {
        Ok(FilterTrailersStatus::Continue)
    }

    /// Called with response headers to be encoded.
    fn on_response_headers(
        &mut self,
        _num_headers: usize,
        _end_of_stream: bool,
        _ops: &dyn ResponseHeadersOps,
    ) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    /// Called with response body to be encoded.
    fn on_response_body(
        &mut self,
        _data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn ResponseBodyOps,
    ) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    /// Called with response trailers to be encoded.
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
    fn on_exchange_complete(&mut self, _ops: &dyn ExchangeCompleteOps) -> Result<()> {
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

/// An interface for manipulating request headers.
pub trait RequestHeadersOps: RequestFlowOps {
    fn request_headers(&self) -> host::Result<HeaderMap>;

    fn request_header(&self, name: &str) -> host::Result<Option<ByteString>>;

    fn set_request_headers(&self, headers: &HeaderMap) -> host::Result<()>;

    fn set_request_header(&self, name: &str, value: &str) -> host::Result<()> {
        self.set_request_header_bytes(name, value.as_bytes())
    }

    fn set_request_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()>;

    fn remove_request_header(&self, name: &str) -> host::Result<()>;
}

/// An interface for manipulating request body.
pub trait RequestBodyOps: RequestFlowOps {
    /// Returns request data received from `Downstream`.
    ///
    /// # Arguments
    ///
    /// * `offset`   - offset to start reading data from.
    /// * `max_size` - maximum size of data to return.
    fn request_data(&self, start: usize, max_size: usize) -> host::Result<ByteString>;

    /// Mutate request data received from `Downstream`.
    ///
    /// # Arguments
    ///
    /// * `change` - transformation to apply to data in the buffer.
    fn mutate_request_data(&self, change: Transform) -> host::Result<()>;
}

/// An interface for manipulating request trailers.
pub trait RequestTrailersOps: RequestFlowOps {
    fn request_trailers(&self) -> host::Result<HeaderMap>;

    fn request_trailer(&self, name: &str) -> host::Result<Option<ByteString>>;

    fn set_request_trailers(&self, trailers: &HeaderMap) -> host::Result<()>;

    fn set_request_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        self.set_request_trailer_bytes(name, value.as_bytes())
    }

    fn set_request_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()>;

    fn remove_request_trailer(&self, name: &str) -> host::Result<()>;
}

/// An interface for changing request flow.
pub trait RequestFlowOps {
    fn resume_request(&self) -> host::Result<()>;

    fn send_response(
        &self,
        status_code: u32,
        headers: &[(&str, &str)],
        body: Option<&[u8]>,
    ) -> host::Result<()>;
}

/// An interface for manipulating response headers.
pub trait ResponseHeadersOps: ResponseFlowOps {
    fn response_headers(&self) -> host::Result<HeaderMap>;

    fn response_header(&self, name: &str) -> host::Result<Option<ByteString>>;

    fn set_response_headers(&self, headers: &HeaderMap) -> host::Result<()>;

    fn set_response_header(&self, name: &str, value: &str) -> host::Result<()> {
        self.set_response_header_bytes(name, value.as_bytes())
    }

    fn set_response_header_bytes(&self, name: &str, value: &[u8]) -> host::Result<()>;

    fn remove_response_header(&self, name: &str) -> host::Result<()>;
}

/// An interface for manipulating response data.
pub trait ResponseBodyOps: ResponseFlowOps {
    /// Returns response data received from `Upstream`.
    ///
    /// # Arguments
    ///
    /// * `offset`   - offset to start reading data from.
    /// * `max_size` - maximum size of data to return.
    fn response_data(&self, start: usize, max_size: usize) -> host::Result<ByteString>;

    /// Mutate response data received from `Upstream`.
    ///
    /// # Arguments
    ///
    /// * `change` - transformation to apply to data in the buffer.
    fn mutate_response_data(&self, change: Transform) -> host::Result<()>;
}

/// An interface for manipulating response trailers.
pub trait ResponseTrailersOps: ResponseFlowOps {
    fn response_trailers(&self) -> host::Result<HeaderMap>;

    fn response_trailer(&self, name: &str) -> host::Result<Option<ByteString>>;

    fn set_response_trailers(&self, headers: &HeaderMap) -> host::Result<()>;

    fn set_response_trailer(&self, name: &str, value: &str) -> host::Result<()> {
        self.set_response_trailer_bytes(name, value.as_bytes())
    }

    fn set_response_trailer_bytes(&self, name: &str, value: &[u8]) -> host::Result<()>;

    fn remove_response_trailer(&self, name: &str) -> host::Result<()>;
}

/// An interface for changing response flow.
pub trait ResponseFlowOps {
    fn resume_response(&self) -> host::Result<()>;
}

/// An interface for operations available in the context of [`on_exchange_complete`]
/// filter invocation.
///
/// [`on_exchange_complete`]: trait.HttpFilter.html#method.on_exchange_complete
pub trait ExchangeCompleteOps {
    // TODO(yskopets): define
}

/// An interface with all available operations over request/response.
pub trait Ops:
    RequestHeadersOps
    + RequestBodyOps
    + RequestTrailersOps
    + ResponseHeadersOps
    + ResponseBodyOps
    + ResponseTrailersOps
    + ExchangeCompleteOps
{
    fn as_request_headers_ops(&self) -> &dyn RequestHeadersOps;

    fn as_request_body_ops(&self) -> &dyn RequestBodyOps;

    fn as_request_trailers_ops(&self) -> &dyn RequestTrailersOps;

    fn as_response_headers_ops(&self) -> &dyn ResponseHeadersOps;

    fn as_response_body_ops(&self) -> &dyn ResponseBodyOps;

    fn as_response_trailers_ops(&self) -> &dyn ResponseTrailersOps;

    fn as_exchange_complete_ops(&self) -> &dyn ExchangeCompleteOps;
}

impl<T> Ops for T
where
    T: RequestHeadersOps
        + RequestBodyOps
        + RequestTrailersOps
        + ResponseHeadersOps
        + ResponseBodyOps
        + ResponseTrailersOps
        + ExchangeCompleteOps,
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

    fn as_exchange_complete_ops(&self) -> &dyn ExchangeCompleteOps {
        self
    }
}

impl dyn Ops {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
