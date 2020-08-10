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

//! `Envoy` `HTTP Client API`.

use std::time::Duration;

use crate::host::{self, Bytes, HeaderMap, HeaderValue};

pub use crate::abi::proxy_wasm::types::HttpRequestHandle as HttpClientRequestHandle;

/// An interface of the `Envoy` `HTTP Client`.
///
/// # Examples
///
/// #### Basic usage of [`HttpClient`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use std::time::Duration;
/// use envoy::host::HttpClient;
///
/// let client = HttpClient::default();
///
/// let request_id = client.send_request(
///     "cluster_name",
///     &[("header", b"value")],
///     Some(b"request body"),
///     &[("trailer", b"value")],
///     Duration::from_secs(5),
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// #### Injecting [`HttpClient`] into a HTTP Filter as a dependency:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::host::HttpClient;
///
/// struct MyHttpFilter<'a> {
///     http_client: &'a dyn HttpClient,
/// }
///
/// impl<'a> MyHttpFilter<'a> {
///     /// Creates a new instance parameterized with a given [`HttpClient`] implementation.
///     pub fn new(http_client: &'a dyn HttpClient) -> Self {
///         MyHttpFilter { http_client }
///     }
///
///     /// Creates a new instance parameterized with the default [`HttpClient`] implementation.
///     pub fn default() -> Self {
///         Self::new(HttpClient::default())
///     }
/// }
/// ```
///
/// #### Sending a request and receiving a response inside a `HTTP Filter`:
///
/// ```
/// # use envoy_sdk as envoy;
/// use std::time::Duration;
/// use envoy::error::format_err;
/// use envoy::extension::{HttpFilter, Result};
/// use envoy::extension::filter::http::{FilterHeadersStatus, RequestHeadersOps, Ops};
/// use envoy::host::HttpClient;
/// use envoy::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};
///
/// struct MyHttpFilter<'a> {
///     http_client: &'a dyn HttpClient,
///
///     active_request: Option<HttpClientRequestHandle>,
/// }
///
/// impl<'a> HttpFilter for MyHttpFilter<'a> {
///     fn on_request_headers(&mut self, _num_headers: usize, ops: &dyn RequestHeadersOps) -> Result<FilterHeadersStatus> {
///         self.http_client.send_request(
///             "cluster_name",
///             &[("header", b"value")],
///             Some(b"request body"),
///             &[("trailer", b"value")],
///             Duration::from_secs(5),
///         )?;
///         Ok(FilterHeadersStatus::StopIteration)  // stop further request processing
///     }
///
///     fn on_http_call_response(
///        &mut self,
///        request: HttpClientRequestHandle,
///        _num_headers: usize,
///        body_size: usize,
///        _num_trailers: usize,
///        filter_ops: &dyn Ops,
///        http_client_ops: &dyn HttpClientResponseOps,
///    ) -> Result<()> {
///        if self.active_request != Some(request) {
///            // don't use `assert!()` to avoid panicing in production code
///            return Err(format_err!("received unexpected response from HttpClient"));
///        }
///        let response_headers = http_client_ops.http_call_response_headers()?;
///        let response_body = http_client_ops.http_call_response_body(0, body_size)?;
/// #      stringify! {
///        ... look into response headers and response body ...
/// #      };
///        filter_ops.resume_request() // resume further request processing
///    }
/// }
/// ```
///
/// [`HttpClient`]: trait.HttpClient.html
pub trait HttpClient {
    /// Sends an HTTP request asynchronously.
    ///
    /// # Arguments
    ///
    /// * `upstream` - name of `Envoy` `Cluster` to send request to.
    /// * `headers`  - request headers
    /// * `body`     - request body
    /// * `trailers` - request trailers
    /// * `timeout`  - request timeout
    ///
    /// # Return value
    ///
    /// opaque [`identifier`][`HttpClientRequestHandle`] of the request sent. Can be used to correlate requests and responses.
    ///
    /// [`HttpClientRequestHandle`]: struct.HttpClientRequestHandle.html
    fn send_request(
        &self,
        upstream: &str,
        headers: &[(&str, &[u8])],
        body: Option<&[u8]>,
        trailers: &[(&str, &[u8])],
        timeout: Duration,
    ) -> host::Result<HttpClientRequestHandle>;
}

impl dyn HttpClient {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn HttpClient {
        &impls::Host
    }
}

/// An interface for accessing data of the HTTP response received by [`HttpClient`].
///
/// [`HttpClient`]: trait.HttpClient.html
pub trait HttpClientResponseOps {
    fn http_call_response_headers(&self) -> host::Result<HeaderMap>;

    fn http_call_response_header(&self, name: &str) -> host::Result<Option<HeaderValue>>;

    fn http_call_response_body(&self, start: usize, max_size: usize) -> host::Result<Bytes>;

    fn http_call_response_trailers(&self) -> host::Result<HeaderMap>;

    fn http_call_response_trailer(&self, name: &str) -> host::Result<Option<HeaderValue>>;
}

impl dyn HttpClientResponseOps {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn HttpClientResponseOps {
        &impls::Host
    }
}

mod impls {
    use std::time::Duration;

    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::{BufferType, MapType};

    use super::{HttpClient, HttpClientRequestHandle, HttpClientResponseOps};
    use crate::host::{self, Bytes, HeaderMap, HeaderValue};

    pub(super) struct Host;

    impl HttpClient for Host {
        fn send_request(
            &self,
            upstream: &str,
            headers: &[(&str, &[u8])],
            body: Option<&[u8]>,
            trailers: &[(&str, &[u8])],
            timeout: Duration,
        ) -> host::Result<HttpClientRequestHandle> {
            hostcalls::dispatch_http_call(upstream, headers, body, trailers, timeout)
        }
    }

    impl HttpClientResponseOps for Host {
        fn http_call_response_headers(&self) -> host::Result<HeaderMap> {
            hostcalls::get_map(MapType::HttpCallResponseHeaders)
        }

        fn http_call_response_header(&self, name: &str) -> host::Result<Option<HeaderValue>> {
            hostcalls::get_map_value(MapType::HttpCallResponseHeaders, name)
        }

        fn http_call_response_body(&self, start: usize, max_size: usize) -> host::Result<Bytes> {
            hostcalls::get_buffer(BufferType::HttpCallResponseBody, start, max_size)
        }

        fn http_call_response_trailers(&self) -> host::Result<HeaderMap> {
            hostcalls::get_map(MapType::HttpCallResponseTrailers)
        }

        fn http_call_response_trailer(&self, name: &str) -> host::Result<Option<HeaderValue>> {
            hostcalls::get_map_value(MapType::HttpCallResponseTrailers, name)
        }
    }
}
