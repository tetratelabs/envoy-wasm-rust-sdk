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

use super::dispatcher::{ContextSelector, VoidContextSelector};
use crate::extension::{Module, Result};

/// Generates the [`_start`] function that will be called by `Envoy` to let
/// WebAssembly module initialize itself.
///
/// [`_start`]: https://github.com/proxy-wasm/spec/blob/master/abi-versions/vNEXT/README.md#_start
///
/// # Examples
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::extension::{self, AccessLogger, NetworkFilter, HttpFilter, InstanceId, ExtensionFactory};
/// #
/// # struct MyHttpFilter;
/// # impl HttpFilter for MyHttpFilter {}
/// #
/// # struct MyHttpFilterFactory;
/// # impl MyHttpFilterFactory {
/// #     fn default() -> extension::Result<Self> { Ok(MyHttpFilterFactory) }
/// # }
/// # impl ExtensionFactory for MyHttpFilterFactory {
/// #     type Extension = MyHttpFilter;
/// #
/// #     const NAME: &'static str = "my.http_filter";
/// #
/// #     fn new_extension(&mut self, instance_id: InstanceId) -> extension::Result<Self::Extension> {
/// #         Ok(MyHttpFilter)
/// #     }
/// # }
/// #
/// # struct MyNetworkFilter;
/// # impl NetworkFilter for MyNetworkFilter {}
/// #
/// # struct MyNetworkFilterFactory;
/// # impl MyNetworkFilterFactory {
/// #     fn default() -> extension::Result<Self> { Ok(MyNetworkFilterFactory) }
/// # }
/// # impl ExtensionFactory for MyNetworkFilterFactory {
/// #     type Extension = MyNetworkFilter;
/// #
/// #     const NAME: &'static str = "my.network_filter";
/// #
/// #     fn new_extension(&mut self, instance_id: InstanceId) -> extension::Result<Self::Extension> {
/// #         Ok(MyNetworkFilter)
/// #     }
/// # }
/// #
/// # struct MyAccessLogger;
/// # impl AccessLogger for MyAccessLogger {
/// #     const NAME: &'static str = "my.access_logger";
/// # }
/// # impl MyAccessLogger {
/// #     fn default() -> extension::Result<Self> { Ok(MyAccessLogger) }
/// # }
/// #
/// use envoy::extension::{entrypoint, Module, Result};
///
/// entrypoint! { initialize } // put initialization logic into a function to make it unit testable
///
/// /// Does one-time initialization.
/// ///
/// /// Returns a registry of extensions provided by this module.
/// fn initialize() -> Result<Module> {
///   // arbitrary initialization steps
///
///   Module::new()
///       .add_http_filter(|_instance_id| MyHttpFilterFactory::default())?
///       .add_network_filter(|_instance_id| MyNetworkFilterFactory::default())?
///       .add_access_logger(|_instance_id| MyAccessLogger::default())
/// }
/// ```
#[macro_export]
macro_rules! entrypoint {
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
            use $crate::extension::{self, Module, Result};
            use $crate::host::log;

            fn init<F>(init_fn: F)
            where
                F: FnOnce() -> Result<Module>,
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
pub fn install(config: Result<Module>) {
    match config {
        Ok(module) => ContextSelector::with_default_ops(module.into()).install(),
        Err(err) => VoidContextSelector::new(err).install(),
    }
}
