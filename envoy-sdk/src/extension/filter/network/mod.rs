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

//! `Envoy` `Network Filter` extension.
//!
//! Creating a new `Network Filter` extension using `Envoy SDK` consists of the following steps:
//!
//! 1. Implement [`NetworkFilter`] trait to define core logic of your extension
//! 2. Implement [`ExtensionFactory`] trait to create new instances of your extension
//! 3. [`Register`] your extension on WebAssembly module start up
//!
//! # Examples
//!
//! #### Basic [`NetworkFilter`]:
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::extension::NetworkFilter;
//!
//! /// My very own `NetworkFilter`.
//! struct MyNetworkFilter;
//!
//! impl NetworkFilter for MyNetworkFilter {}
//! ```
//!
//! #### `ExtensionFactory` for `MyNetworkFilter` instances:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::NetworkFilter;
//! #
//! # /// My very own `NetworkFilter`.
//! # struct MyNetworkFilter;
//! #
//! # impl NetworkFilter for MyNetworkFilter {}
//! #
//! use envoy::extension::{ExtensionFactory, InstanceId, Result};
//!
//! /// `ExtensionFactory` for `MyNetworkFilter`.
//! struct MyNetworkFilterFactory;
//!
//! impl ExtensionFactory for MyNetworkFilterFactory {
//!     type Extension = MyNetworkFilter;
//!
//!     fn name() -> &'static str { "my_network_filter" }
//!
//!     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
//!         Ok(MyNetworkFilter)
//!     }
//! }
//! ```
//!
//! #### Registration of `MyNetworkFilter` on start up:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::NetworkFilter;
//! #
//! # /// My very own `NetworkFilter`.
//! # struct MyNetworkFilter;
//! # impl NetworkFilter for MyNetworkFilter {}
//! #
//! # use envoy::extension::{ExtensionFactory, InstanceId, self};
//! #
//! # /// `ExtensionFactory` for `MyNetworkFilter`.
//! # struct MyNetworkFilterFactory;
//! #
//! # impl ExtensionFactory for MyNetworkFilterFactory {
//! #     type Extension = MyNetworkFilter;
//! #
//! #     fn name() -> &'static str { "my_network_filter" }
//! #
//! #     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
//! #         Ok(MyNetworkFilter)
//! #     }
//! # }
//! use envoy::extension::{entrypoint, Module, Result};
//!
//! entrypoint! { initialize } // put initialization logic into a function to make it unit testable
//!
//! fn initialize() -> Result<Module> {
//!     Module::new()
//!         .add_network_filter(|_instance_id| Ok(MyNetworkFilterFactory))
//! }
//! ```
//!
//! [`NetworkFilter`]: trait.NetworkFilter.html
//! [`ExtensionFactory`]: ../../factory/trait.ExtensionFactory.html
//! [`Register`]: ../../../macro.entrypoint.html

use crate::abi::proxy_wasm::types::{Action, PeerType};
use crate::extension::Result;
use crate::host::http::client::{HttpClientRequestHandle, HttpClientResponseOps};
use crate::host::{self, ByteString};

pub(crate) use self::context::{NetworkFilterContext, VoidNetworkFilterContext};

mod context;
mod ops;

/// Return codes for [`on_downstream_data`] and [`on_upstream_data`] filter
/// invocations.
///
/// `Envoy` bases further filter invocations on the return code of the
/// previous filter.
///
/// [`on_downstream_data`]: trait.NetworkFilter.html#method.on_downstream_data
/// [`on_upstream_data`]: trait.NetworkFilter.html#method.on_upstream_data
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum FilterStatus {
    /// Continue filter chain iteration.
    Continue = 0,
    /// Do not iterate to any of the remaining filters in the chain.
    ///
    /// **WARNING**: At the moment, `Envoy` doesn't yet implement [`ABI`] that
    /// would allow to resume filter iteration.
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_resume_downstream
    StopIteration = 1,
}

impl FilterStatus {
    pub(self) fn as_action(&self) -> Action {
        match self {
            FilterStatus::Continue => Action::Continue,
            FilterStatus::StopIteration => Action::Pause,
        }
    }
}

/// An interface of the `Envoy` `Network Filter` extension.
///
/// `Network Filter` operates on a single TCP connection.
///
/// A dedicated `Network Filter` instance is created for every connection handled by `Envoy`.
///
/// Consequently, state of a single connection can be stored inside `Network Filter` itself.
///
/// # Examples
///
/// #### Basic `Network Filter`:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::extension::{NetworkFilter, Result};
/// use envoy::extension::filter::network::FilterStatus;
/// use envoy::host::log;
///
/// /// My very own `NetworkFilter`.
/// struct MyNetworkFilter;
///
/// impl NetworkFilter for MyNetworkFilter {
///     fn on_new_connection(&mut self) -> Result<FilterStatus> {
///         log::info!("a new connection has been established");
///         Ok(FilterStatus::Continue)
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
/// only the affected TCP connection by closing it gracefully.
///
/// For comparison, if the extension choose to panic, this will, at best, affect all ongoing TCP connections
/// handled by that extension, and, at worst, will crash `Envoy` entirely (as of July 2020).
///
/// [`Result::Err(x)`]: https://doc.rust-lang.org/core/result/enum.Result.html#variant.Err
pub trait NetworkFilter {
    /// Called when a connection is first established.
    ///
    /// Filters should do one time long term processing that needs to be done when a connection is
    /// established. Filter chain iteration can be stopped if needed.
    ///
    /// # Return value
    ///
    /// [`FilterStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterStatus`]: enum.FilterStatus.html
    fn on_new_connection(&mut self) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    /// Called when data is read on the downstream connection.
    ///
    /// # Arguments
    ///
    /// * `data_size`     - size of data accumulated in the buffer.
    /// * `end_of_stream` - supplies whether this is the last byte on the connection. This will only
    ///                     be set if the connection has half-close semantics enabled.
    /// * `ops`           - a [`trait object`][`DownstreamDataOps`] through which `Network Filter` can
    ///                     manipulate data in the read buffer.
    ///
    /// # Return value
    ///
    /// [`FilterStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterStatus`]: enum.FilterStatus.html
    /// [`DownstreamDataOps`]: trait.DownstreamDataOps.html
    fn on_downstream_data(
        &mut self,
        _data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn DownstreamDataOps,
    ) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    /// Called when downstream connection is closed.
    ///
    /// # Arguments
    ///
    /// * `peer_type` - supplies who closed the connection (either the remote party or `Envoy` itself).
    fn on_downstream_close(
        &mut self,
        _peer_type: PeerType,
        _ops: &dyn DownstreamCloseOps,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when data is to be written on the connection.
    ///
    /// # Arguments
    ///
    /// * `data_size`     - size of data accumulated in the write buffer.
    /// * `end_of_stream` - supplies whether this is the last byte to write on the connection.
    /// * `ops`           - a [`trait object`][`UpstreamDataOps`] through which `Network Filter` can
    ///                     manipulate data in the write buffer.
    ///
    /// # Return value
    ///
    /// [`FilterStatus`] telling `Envoy` how to manage further filter iteration.
    ///
    /// [`FilterStatus`]: enum.FilterStatus.html
    /// [`UpstreamDataOps`]: trait.UpstreamDataOps.html
    fn on_upstream_data(
        &mut self,
        _data_size: usize,
        _end_of_stream: bool,
        _ops: &dyn UpstreamDataOps,
    ) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    /// Called when upstream connection is closed.
    ///
    /// # Arguments
    ///
    /// * `peer_type` - supplies who closed the connection (either the remote party or `Envoy` itself).
    fn on_upstream_close(
        &mut self,
        _peer_type: PeerType,
        _ops: &dyn UpstreamCloseOps,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when TCP connection is complete.
    ///
    /// This moment happens before `Access Loggers` get called.
    fn on_connection_complete(&mut self, _ops: &dyn ConnectionCompleteOps) -> Result<()> {
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
    /// * `filter_ops`      - a [`trait object`][`Ops`] through which `Network Filter` can manipulate data
    ///                       of the connection it proxies.
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

/// An interface for manipulating data in the read buffer from `Downstream`.
pub trait DownstreamDataOps {
    /// Returns data in the read buffer from `Downstream`.
    ///
    /// # Arguments
    ///
    /// * `offset`   - offset to start reading data from.
    /// * `max_size` - maximum size of data to return.
    fn downstream_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString>;
}

/// An interface for manipulating data received from `Upstream`
/// before they reach the write buffer for `Downstream`.
pub trait UpstreamDataOps {
    /// Returns data received from `Upstream`.
    ///
    /// # Arguments
    ///
    /// * `offset`   - offset to start reading data from.
    /// * `max_size` - maximum size of data to return.
    fn upstream_data(&self, offset: usize, max_size: usize) -> host::Result<ByteString>;
}

/// An interface for operations available in the context of [`on_downstream_close`]
/// filter invocation.
///
/// [`on_downstream_close`]: trait.NetworkFilter.html#method.on_downstream_close
pub trait DownstreamCloseOps {
    // TODO(yskopets): TBD
}

/// An interface for operations available in the context of [`on_upstream_close`]
/// filter invocation.
///
/// [`on_upstream_close`]: trait.NetworkFilter.html#method.on_upstream_close
pub trait UpstreamCloseOps {
    // TODO(yskopets): TBD
}

/// An interface for operations available in the context of [`on_connection_complete`]
/// filter invocation.
///
/// [`on_connection_complete`]: trait.NetworkFilter.html#method.on_connection_complete
pub trait ConnectionCompleteOps {
    // TODO(yskopets): TBD
}

/// An interface for manipulating data in both read and write buffers.
pub trait Ops:
    DownstreamDataOps + UpstreamDataOps + DownstreamCloseOps + UpstreamCloseOps + ConnectionCompleteOps
{
    fn as_downstream_data_ops(&self) -> &dyn DownstreamDataOps;

    fn as_upstream_data_ops(&self) -> &dyn UpstreamDataOps;

    fn as_downstream_close_ops(&self) -> &dyn DownstreamCloseOps;

    fn as_upstream_close_ops(&self) -> &dyn UpstreamCloseOps;

    fn as_connection_complete_ops(&self) -> &dyn ConnectionCompleteOps;
}

impl<T> Ops for T
where
    T: DownstreamDataOps
        + UpstreamDataOps
        + DownstreamCloseOps
        + UpstreamCloseOps
        + ConnectionCompleteOps,
{
    fn as_downstream_data_ops(&self) -> &dyn DownstreamDataOps {
        self
    }

    fn as_upstream_data_ops(&self) -> &dyn UpstreamDataOps {
        self
    }

    fn as_downstream_close_ops(&self) -> &dyn DownstreamCloseOps {
        self
    }

    fn as_upstream_close_ops(&self) -> &dyn UpstreamCloseOps {
        self
    }

    fn as_connection_complete_ops(&self) -> &dyn ConnectionCompleteOps {
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
