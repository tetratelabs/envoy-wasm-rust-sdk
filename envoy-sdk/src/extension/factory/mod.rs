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

//! `Envoy` `Extension Factory`.
//!
//! [`ExtensionFactory`] is responsible for handling extension configuration
//! and creating new instances of extension.
//!
//! # Examples
//!
//! #### Basic [`ExtensionFactory`]:
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::HttpFilter;
//! #
//! # /// My very own `HttpFilter`.
//! # struct MyHttpFilter;
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
//!     const NAME: &'static str = "my_http_filter";
//!
//!     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
//!         Ok(MyHttpFilter)
//!     }
//! }
//! ```
//!
//! [`ExtensionFactory`]: trait.ExtensionFactory.html

use crate::extension::{factory, InstanceId, Result};
use crate::host::{self, ByteString};

pub(crate) use self::context::ExtensionFactoryContext;

mod context;
mod ops;

/// Possible responses to the request to (re-)configure the extension.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConfigStatus {
    /// Extension has accepted the new configuration.
    Accepted,
    /// Extension has rejected the new configuration.
    Rejected,
}

impl ConfigStatus {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            ConfigStatus::Accepted => true,
            ConfigStatus::Rejected => false,
        }
    }
}

/// Possible responses to the request to drain the extension.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DrainStatus {
    /// Extension is being drained and cannot be removed just yet.
    Ongoing,
    /// Extension has been drained and can be removed now.
    Complete,
}

impl DrainStatus {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            DrainStatus::Ongoing => false,
            DrainStatus::Complete => true,
        }
    }
}

/// An interface of the `Envoy` `Extension Factory`.
///
/// [`ExtensionFactory`] is responsible for
/// * handling extension configuration,
/// * owning state shared by all extension instances,
/// * creating new instances of extension, injecting their dependencies and shared state.
///
/// At the moment, [`ExtensionFactory`] can be used for [`HttpFilter`] and [`NetworkFilter`]
/// extensions.
///
/// [`AccessLogger`] extension has a different lifecycle and therefore manages its
/// configuration differently.
///
/// # Examples
///
/// #### Basic [`ExtensionFactory`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::extension::HttpFilter;
/// #
/// # /// My very own `HttpFilter`.
/// # struct MyHttpFilter;
/// # impl HttpFilter for MyHttpFilter {}
/// #
/// use envoy::extension::{ExtensionFactory, InstanceId, Result};
///
/// /// `ExtensionFactory` for `MyHttpFilter`.
/// struct MyHttpFilterFactory;
///
/// impl ExtensionFactory for MyHttpFilterFactory {
///     type Extension = MyHttpFilter;
///
///     const NAME: &'static str = "my_http_filter";
///
///     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
///         Ok(MyHttpFilter)
///     }
/// }
/// ```
///
/// #### Handling extension configuration:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::extension::HttpFilter;
/// #
/// # /// My very own `HttpFilter`.
/// # struct MyHttpFilter {
/// #     config: std::rc::Rc<String>,
/// # }
/// # impl HttpFilter for MyHttpFilter {}
/// # impl MyHttpFilter {
/// #     fn new(config: std::rc::Rc<String>) -> Self {
/// #         MyHttpFilter { config }
/// #     }
/// # }
/// #
/// use std::rc::Rc;
/// use envoy::extension::{factory, ConfigStatus, ExtensionFactory, InstanceId, Result};
/// use envoy::host::ByteString;
///
/// /// `ExtensionFactory` for `MyHttpFilter`.
/// struct MyHttpFilterFactory {
///     // This example shows how multiple filter instances could share
///     // the same configuration.
///     config: Rc<String>,
/// }
///
/// impl ExtensionFactory for MyHttpFilterFactory {
///     type Extension = MyHttpFilter;
///
///     const NAME: &'static str = "my_http_filter";
///
///     /// Called when extension is being (re-)configured on `Envoy Listener` update.
///     fn on_configure(&mut self, config: ByteString, ops: &dyn factory::ConfigureOps) -> Result<ConfigStatus> {
///         let config = if config.is_empty() { String::default() } else {
///             String::from_utf8(config.into_bytes())?
///         };
///         self.config = Rc::new(config);
///         Ok(ConfigStatus::Accepted)
///     }
///
///     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
///         Ok(MyHttpFilter::new(Rc::clone(&self.config)))
///     }
/// }
/// ```
///
/// #### Sharing `Stats` between filter instances:
///
/// ```
/// # use envoy_sdk as envoy;
/// use std::rc::Rc;
/// use envoy::extension::{factory, ConfigStatus, ExtensionFactory, InstanceId, Result};
/// use envoy::host::stats::{Counter, Stats};
///
/// /// Stats shared between multiple filter instances.
/// pub struct MyStats {
///     requests_total: Box<dyn Counter>,
/// }
///
/// # use envoy::extension::HttpFilter;
/// #
/// # /// My very own `HttpFilter`.
/// # struct MyHttpFilter {
/// #     stats: std::rc::Rc<MyStats>,
/// # }
/// # impl HttpFilter for MyHttpFilter {}
/// # impl MyHttpFilter {
/// #     fn new(stats: std::rc::Rc<MyStats>) -> Self {
/// #         MyHttpFilter { stats }
/// #     }
/// # }
/// #
/// /// `ExtensionFactory` for `MyHttpFilter`.
/// struct MyHttpFilterFactory {
///     // This example shows how multiple filter instances could share stats.
///     stats: Rc<MyStats>,
/// }
///
/// impl ExtensionFactory for MyHttpFilterFactory {
///     type Extension = MyHttpFilter;
///
///     const NAME: &'static str = "my_http_filter";
///
///     fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
///         Ok(MyHttpFilter::new(Rc::clone(&self.stats)))
///     }
/// }
///
/// impl MyHttpFilterFactory {
///     /// Creates a new factory.
///     pub fn new(stats: &dyn Stats) -> Result<Self> {
///         let my_stats = MyStats{
///             requests_total: stats.counter("examples.http_filter.requests_total")?,
///         };
///         Ok(MyHttpFilterFactory {
///             stats: Rc::new(my_stats),
///         })
///     }
///
///     /// Creates a new factory bound to the actual `Envoy` `ABI`.
///     pub fn default() -> Result<Self> {
///         Self::new(Stats::default())
///     }
/// }
/// ```
///
/// [`ExtensionFactory`]: trait.ExtensionFactory.html
/// [`HttpFilter`]: ../filter/http/trait.HttpFilter.html
/// [`NetworkFilter`]: ../filter/network/trait.NetworkFilter.html
/// [`AccessLogger`]: ../access_logger/trait.AccessLogger.html
pub trait ExtensionFactory {
    type Extension;

    /// Name the extension should be referred to in `Envoy` configuration.
    const NAME: &'static str;

    /// Called when extension is being (re-)configured on `Envoy Listener` update.
    ///
    /// # Arguments
    ///
    /// * `_config` - configuration.
    /// * `_ops`    - a [`trait object`][`ConfigureOps`] with operations available in this context.
    ///
    /// # Return value
    ///
    /// [`ConfigStatus`] telling `Envoy` whether configuration has been successfully applied.
    ///
    /// [`ConfigStatus`]: enum.ConfigStatus.html
    /// [`ConfigureOps`]: trait.ConfigureOps.html
    fn on_configure(
        &mut self,
        _config: ByteString,
        _ops: &dyn factory::ConfigureOps,
    ) -> Result<ConfigStatus> {
        Ok(ConfigStatus::Accepted)
    }

    /// Called to create a new instance of the extension, e.g. [`HttpFilter`] or [`NetworkFilter`].
    ///
    /// # Arguments
    ///
    /// * `instance_id` - opaque identifier of the extension instance.
    ///
    /// # Return value
    ///
    /// a new instance of the extension.
    ///
    /// [`HttpFilter`]: ../filter/http/trait.HttpFilter.html
    /// [`NetworkFilter`]: ../filter/network/trait.NetworkFilter.html
    fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension>;

    /// Called when `ExtensionFactory` is about to be destroyed.
    ///
    /// # Return value
    ///
    /// [`DrainStatus`] telling `Envoy` whether `ExtensionFactory` has already been drained
    /// and can be now removed safely.
    ///
    /// [`DrainStatus`]: enum.DrainStatus.html
    fn on_drain(&mut self) -> Result<DrainStatus> {
        Ok(DrainStatus::Complete)
    }
}

/// An interface for accessing extension config.
pub(crate) trait ContextOps {
    /// Returns extension config.
    fn configuration(&self) -> host::Result<ByteString>;
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
/// [`on_configure`]: trait.ExtensionFactory.html#method.on_configure
pub trait ConfigureOps {}

/// An interface for acknowledging `Envoy` that [`ExtensionFactory`] has been drained.
///
/// [`ExtensionFactory`]: trait.ExtensionFactory.html
pub trait DrainOps {
    /// Acknowledges `Envoy` that extension has been drained and can be safely removed now.
    fn done(&self) -> host::Result<()>;
}

#[doc(hidden)]
pub trait Ops: ConfigureOps + DrainOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps;

    fn as_done_ops(&self) -> &dyn DrainOps;
}

impl<T> Ops for T
where
    T: ConfigureOps + DrainOps,
{
    fn as_configure_ops(&self) -> &dyn ConfigureOps {
        self
    }

    fn as_done_ops(&self) -> &dyn DrainOps {
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
