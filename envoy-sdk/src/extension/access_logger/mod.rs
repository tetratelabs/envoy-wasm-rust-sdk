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

//! `Envoy` `Access Logger API`.

use crate::abi::proxy_wasm::types::Bytes;

use crate::extension::{ConfigStatus, Result};
use crate::host;
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};

pub(crate) use self::context::AccessLoggerContext;

mod context;
mod ops;

/// An interface of the `Envoy` `Access Logger` extension.
///
/// In contrast to [`HttpFilter`] and [`NetworkFilter`] that only operate on a single
/// HTTP stream and TCP connection respectively, `Access Logger` operates on multiple
/// HTTP streams or TCP connections.
///
/// `Access Logger` in `Envoy` is a stateful object.
///
/// **NOTE: This trait MUST NOT panic**. If a logger invocation cannot proceed
/// normally, it should return [`Result::Err(x)`]. In that case, [`Envoy SDK`] will be able to handle
/// the error gracefully.
/// For comparison, if the extension chooses to panic, this will, at best, affect all ongoing HTTP requests
/// / TCP connections handled by that extension, and, at worst, will crash `Envoy` entirely (as of July 2020).
///
/// [`HttpFilter`]: ../filter/http/trait.HttpFilter.html
/// [`NetworkFilter`]: ../filter/network/trait.NetworkFilter.html
/// [`Result::Err(x)`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
/// [`Envoy SDK`]: https://docs.rs/envoy-sdk
pub trait AccessLogger {
    /// Name the extension should be referred to in `Envoy` configuration.
    const NAME: &'static str;

    fn on_configure(
        &mut self,
        _configuration_size: usize,
        _logger_ops: &dyn ConfigureOps,
    ) -> Result<ConfigStatus> {
        Ok(ConfigStatus::Accepted)
    }

    /// Called when HTTP request or TCP connection is complete.
    ///
    /// # Arguments
    ///
    /// * `logger_ops` - a [`trait object`][`LogOps`] through which `Access Logger` can access
    ///                  data of the HTTP stream or TCP connection that needs to be logged.
    ///
    /// [`LogOps`]: trait.LogOps.html
    fn on_log(&mut self, _logger_ops: &dyn LogOps) -> Result<()> {
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
    /// * `http_client_ops` - a [`trait object`][`HttpClientResponseOps`] through which `Access Logger` can access
    ///                       data of the response received by [`HttpClient`], including headers, body and trailers.
    ///
    /// [`HttpClient`]: ../../host/http/client/trait.HttpClient.html
    /// [`HttpClientResponseOps`]: ../../host/http/client/trait.HttpClientResponseOps.html
    /// [`Ops`]: trait.Ops.html
    fn on_http_call_response(
        &mut self,
        _request_id: HttpClientRequestHandle,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        _http_client_ops: &dyn HttpClientResponseOps,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait ConfigureOps {
    fn configuration(&self) -> host::Result<Option<Bytes>>;
}

pub trait LogOps {
    fn request_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn request_header(&self, name: &str) -> host::Result<Option<String>>;

    fn response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn response_header(&self, name: &str) -> host::Result<Option<String>>;

    fn response_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn response_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn stream_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;
}

pub trait Ops: ConfigureOps + LogOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps;

    fn as_log_ops(&self) -> &dyn LogOps;
}

impl<T> Ops for T
where
    T: ConfigureOps + LogOps,
{
    fn as_configure_ops(&self) -> &dyn ConfigureOps {
        self
    }

    fn as_log_ops(&self) -> &dyn LogOps {
        self
    }
}

impl dyn Ops {
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
