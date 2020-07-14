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

use super::dispatcher::{ContextSelector, InvalidVmContext};
use crate::extension::{Registry, Result};

/// Generates the [`_start`] function that will be called by `Envoy` to let
/// WebAssembly module initialize itself.
///
/// [`_start`]: https://github.com/proxy-wasm/spec/blob/master/abi-versions/vNEXT/README.md#_start
///
/// # Examples
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::extension::{access_logger, filter::network, filter::http, InstanceId, Result};
/// #
/// # struct MyHttpFilter;
/// # impl http::Filter for MyHttpFilter {}
/// #
/// # struct MyHttpFilterFactory;
/// # impl MyHttpFilterFactory {
/// #     fn default() -> Result<Self> { Ok(MyHttpFilterFactory) }
/// # }
/// # impl envoy::extension::Factory for MyHttpFilterFactory {
/// #     type Extension = MyHttpFilter;
/// #
/// #     const NAME: &'static str = "my.http_filter";
/// #
/// #     fn new_extension(&mut self, instance_id: InstanceId) -> Result<Self::Extension> {
/// #         Ok(MyHttpFilter)
/// #     }
/// # }
/// #
/// # struct MyNetworkFilter;
/// # impl network::Filter for MyNetworkFilter {}
/// #
/// # struct MyNetworkFilterFactory;
/// # impl MyNetworkFilterFactory {
/// #     fn default() -> Result<Self> { Ok(MyNetworkFilterFactory) }
/// # }
/// # impl envoy::extension::Factory for MyNetworkFilterFactory {
/// #     type Extension = MyNetworkFilter;
/// #
/// #     const NAME: &'static str = "my.network_filter";
/// #
/// #     fn new_extension(&mut self, instance_id: InstanceId) -> Result<Self::Extension> {
/// #         Ok(MyNetworkFilter)
/// #     }
/// # }
/// #
/// # struct MyAccessLogger;
/// # impl access_logger::Logger for MyAccessLogger {
/// #     const NAME: &'static str = "my.access_logger";
/// # }
/// # impl MyAccessLogger {
/// #     fn default() -> Result<Self> { Ok(MyAccessLogger) }
/// # }
/// #
/// use envoy::{extension, extension::Registry, on_module_load};
///
/// on_module_load! { initialize } // put initialization logic into a function to make it unit testable
///
/// /// Does one-time initialization.
/// ///
/// /// Returns a registry of extensions provided by this module.
/// fn initialize() -> extension::Result<Registry> {
///   // arbitrary initialization steps
///
///   Registry::new()
///       .add_http_filter(|_instance_id| MyHttpFilterFactory::default())?
///       .add_network_filter(|_instance_id| MyNetworkFilterFactory::default())?
///       .add_access_logger(|_instance_id| MyAccessLogger::default())
/// }
/// ```
#[macro_export]
macro_rules! on_module_load {
    // Apparently, Rust toolchain doesn't handle well exported name `_start`
    // when a package is compiled to targets other than `wasm32-unknown-unknown`.
    // Specifically, linking issues have been observed with targets `wasm32-wasi`
    // and `x86_64-unknown-linux-gnu`, which blocks unit testing.
    // Therefore, only use export name `_start` when in the context of target
    // `wasm32-unknown-unknown`.
    ($init_fn:expr) => {
        #[cfg_attr(
            all(
                target_arch = "wasm32",
                target_vendor = "unknown",
                target_os = "unknown"
            ),
            export_name = "_start"
        )]
        #[no_mangle]
        extern "C" fn start() {
            use $crate::extension::{self, Registry, Result};
            use $crate::host::log;

            fn init<F>(init_fn: F)
            where
                F: FnOnce() -> Result<Registry>,
            {
                // Apparently, `proxy_wasm` uses `set_log_level`
                // to set a custom panic handler that will log panics using Envoy Log API.
                // To be sure that panics will always be set,
                // we call `set_log_level` ourselves instead of leaving it up to a user.
                log::set_max_level(log::LogLevel::Info);

                // Call the init callback provided as an argument.
                extension::install(init_fn());
            }

            init($init_fn);
        }
    };
}

#[doc(hidden)]
pub fn install(config: Result<Registry>) {
    match config {
        Ok(registry) => ContextSelector::with_default_ops(registry.into()).install(),
        Err(err) => InvalidVmContext::install(err),
    }
}
