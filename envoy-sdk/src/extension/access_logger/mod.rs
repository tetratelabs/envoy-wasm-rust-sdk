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

//! `Envoy` `Access Logger` extension.
//!
//! Creating a new `Access Logger` extension using `Envoy SDK` consists of the following steps:
//!
//! 1. Implement [`AccessLogger`] trait to define core logic of your extension
//! 2. [`Register`] your extension on WebAssembly module start up
//!
//! # Examples
//!
//! #### Basic [`AccessLogger`]:
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::extension::AccessLogger;
//!
//! /// My very own `AccessLogger`.
//! struct MyAccessLogger;
//!
//! impl AccessLogger for MyAccessLogger {
//!     const NAME: &'static str = "my_access_logger";
//! }
//! ```
//!
//! #### Registration of `MyAccessLogger` on start up:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::AccessLogger;
//! #
//! # /// My very own `AccessLogger`.
//! # struct MyAccessLogger;
//! #
//! # impl AccessLogger for MyAccessLogger {
//! #     const NAME: &'static str = "my_access_logger";
//! # }
//! #
//! use envoy::extension::{entrypoint, Module, Result};
//!
//! entrypoint! { initialize } // put initialization logic into a function to make it unit testable
//!
//! fn initialize() -> Result<Module> {
//!     Module::new()
//!         .add_access_logger(|_instance_id| Ok(MyAccessLogger))
//! }
//! ```
//!
//! [`AccessLogger`]: trait.AccessLogger.html
//! [`Register`]: ../../macro.entrypoint.html

use crate::extension::{ConfigStatus, DrainStatus, Result};
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};
use crate::host::{self, Bytes, HeaderMap, HeaderValue, StreamInfo};

pub(crate) use self::context::AccessLoggerContext;

mod context;
mod ops;

/// An interface of the `Envoy` `Access Logger` extension.
///
/// In contrast to [`HttpFilter`] and [`NetworkFilter`] that only operate on a single
/// HTTP stream and TCP connection respectively, `Access Logger` operates on multiple
/// HTTP streams or TCP connections.
///
/// # Examples
///
/// #### Basic `AccessLogger`:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::extension::{AccessLogger, Result};
/// use envoy::extension::access_logger::LogOps;
/// use envoy::host::{Bytes, log};
///
/// /// My very own `AccessLogger`.
/// struct MyAccessLogger;
///
/// impl AccessLogger for MyAccessLogger {
///     const NAME: &'static str = "my_access_logger";
///
///     fn on_log(&mut self, logger_ops: &dyn LogOps) -> Result<()> {
///         let upstream_address = logger_ops.stream_info().upstream().address()?
///             .unwrap_or_else(|| "<unknown>".into());
///         log::info!("upstream.address : {}", upstream_address);
///         Ok(())
///     }    
/// }
/// ```
///
/// # NOTE
///
/// **This trait MUST NOT panic!**
///
/// If a logger invocation cannot proceed normally, it should return [`Result::Err(x)`].
/// In that case, `Envoy SDK` will be able to handle the error gracefully.
///
/// For comparison, if the extension chooses to panic, this will, at best, affect all ongoing HTTP requests
/// / TCP connections handled by that extension, and, at worst, will crash `Envoy` entirely (as of July 2020).
///
/// [`HttpFilter`]: ../filter/http/trait.HttpFilter.html
/// [`NetworkFilter`]: ../filter/network/trait.NetworkFilter.html
/// [`Result::Err(x)`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
pub trait AccessLogger {
    /// Name the extension should be referred to in `Envoy` configuration.
    const NAME: &'static str;

    /// Called when `Access Logger` is being (re-)configured.
    ///
    /// # Arguments
    ///
    /// * `_config` - configuration.
    /// * `_ops`    - a [`trait object`][`ConfigureOps`] through which `Access Logger` can access
    ///               its configuration.
    ///
    /// # Return value
    ///
    /// [`ConfigStatus`] telling `Envoy` whether configuration has been successfully applied.
    ///
    /// [`ConfigStatus`]: ../factory/enum.ConfigStatus.html
    /// [`ConfigureOps`]: trait.ConfigureOps.html
    fn on_configure(&mut self, _config: Bytes, _ops: &dyn ConfigureOps) -> Result<ConfigStatus> {
        Ok(ConfigStatus::Accepted)
    }

    /// Called when HTTP request or TCP connection is complete.
    ///
    /// # Arguments
    ///
    /// * `ops` - a [`trait object`][`LogOps`] through which `Access Logger` can access
    ///           data of the HTTP stream or TCP connection that is being logged.
    ///
    /// [`LogOps`]: trait.LogOps.html
    fn on_log(&mut self, _ops: &dyn LogOps) -> Result<()> {
        Ok(())
    }

    /// Called when `Access Logger` is about to be destroyed.
    ///
    /// # Return value
    ///
    /// [`DrainStatus`] telling `Envoy` whether `Access Logger` has already been drained
    /// and can be now removed safely.
    ///
    /// [`DrainStatus`]: ../factory/enum.DrainStatus.html
    fn on_drain(&mut self) -> Result<DrainStatus> {
        Ok(DrainStatus::Complete)
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

/// An interface for accessing extension config.
pub trait ContextOps {
    /// Returns extension config.
    fn configuration(&self) -> host::Result<Bytes>;
}

impl dyn ContextOps {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn ContextOps {
        &ops::Host
    }
}

/// An interface for operations available in the context of [`on_configure`]
/// invocation.
///
/// [`on_configure`]: trait.AccessLogger.html#method.on_configure
pub trait ConfigureOps {}

/// An interface for acknowledging `Envoy` that `AccessLogger` has been drained.
///
/// [`AccessLogger`]: trait.AccessLogger.html
pub trait DrainOps {
    /// Acknowledges `Envoy` that extension has been drained and can be safely removed now.
    fn done(&self) -> host::Result<()>;
}

/// An interface for accessing data of the HTTP stream or TCP connection that is being logged.
pub trait LogOps {
    fn request_headers(&self) -> host::Result<HeaderMap>;

    fn request_header(&self, name: &str) -> host::Result<Option<HeaderValue>>;

    fn response_headers(&self) -> host::Result<HeaderMap>;

    fn response_header(&self, name: &str) -> host::Result<Option<HeaderValue>>;

    fn response_trailers(&self) -> host::Result<HeaderMap>;

    fn response_trailer(&self, name: &str) -> host::Result<Option<HeaderValue>>;

    fn stream_info<'a>(&'a self) -> &'a dyn StreamInfo;
}

#[doc(hidden)]
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
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
