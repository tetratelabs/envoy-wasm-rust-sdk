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

use std::rc::Rc;

use super::ContextFactoryHashMap;

use crate::abi::proxy_wasm_ext;
use crate::abi::proxy_wasm_ext::traits::{Context, RootContext};
use crate::error::format_err;
use crate::extension::error::ConfigurationError;
use crate::extension::error::ErrorSink;
use crate::extension::{Error, Result};
use crate::host::error::function;
use crate::host::stream_info::Service;

pub(crate) struct ContextSelector<'a> {
    factories: ContextFactoryHashMap,
    stream_info: &'a dyn Service,
}

impl<'a> ContextSelector<'a> {
    pub fn new(factories: ContextFactoryHashMap, stream_info: &'a dyn Service) -> Self {
        ContextSelector {
            factories,
            stream_info,
        }
    }

    pub fn with_default_ops(factories: ContextFactoryHashMap) -> Self {
        Self::new(factories, Service::default())
    }

    fn new_root_context(&mut self, context_id: u32) -> Result<Box<dyn RootContext>> {
        let name = match self.stream_info.get_property(vec!["plugin_root_id"])? {
            Some(bytes) => String::from_utf8(bytes).map_err(|e| {
                function("env", "proxy_get_property").into_parse_error(format_err!(
                    "value of property \"{}\" is not a valid UTF-8 string: {:?}",
                    "plugin_root_id",
                    e
                ))
            })?,
            None => String::default(),
        };
        if let Some(root_context_factory) = self.factories.get_mut(&name) {
            return root_context_factory(context_id);
        }
        if name == "" && self.factories.keys().len() == 1 {
            if let Some(root_context_factory) = self.factories.values_mut().next() {
                return root_context_factory(context_id);
            }
        }
        Err(ConfigurationError::UnknownExtension {
            requested: name,
            available: self.factories.keys().cloned().collect(),
        }
        .into())
    }
}

impl ContextSelector<'static> {
    pub fn install(mut self) {
        proxy_wasm_ext::set_root_context(move |context_id| {
            // At the moment, `wasm32-unknown-unknown` and `wasm32-wasi` targets
            // do not support stack unwinding.
            // Consequently, in the case of a panic, memory on heap will not be released.
            // Which leaves Envoy no choice but to deem the VM unsafe to use any longer.
            // Even worse, at the moment, Envoy simply crashes whenever a panic happens
            // inside a WebAssembly module.
            // That is why, instead of raising a panic in here, we memorize the error
            // with the intent to report it later in a manner that won't crash Envoy.
            // Specifically, we're relying on the fact that every `proxy_on_context_create`
            // call will be followed by `proxy_on_configure` where we can legally
            // report back to Envoy that configuration is not valid.
            self.new_root_context(context_id)
                .unwrap_or_else(|e| Box::new(VoidRootContext::with_default_ops(e)))
        });
    }
}

/// Fake `Proxy Wasm` [`RootContext`] that is used to postpone error handling
/// until a proper moment in the extension lifecycle.
///
/// E.g., if an error occurres inside [`proxy_on_context_create`] callback
/// where an Extension Factory instance is supposed to be created,
/// we cannot reject invalid Envoy configuration right away - `Envoy` doesn't expect it
/// at this point.
///
/// Instead, we have to memorize the error and wait until [`proxy_on_configure`]
/// callback when it will be possible to signal back that configuration is not valid.
///
/// [`RootContext`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/traits/trait.RootContext.html
/// [`proxy_on_context_create`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_context_create
/// [`proxy_on_configure`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_configure
struct VoidRootContext<'a> {
    err: Error,
    error_sink: &'a dyn ErrorSink,
}

impl<'a> VoidRootContext<'a> {
    fn new(err: Error, error_sink: &'a dyn ErrorSink) -> Self {
        VoidRootContext { err, error_sink }
    }

    fn with_default_ops(err: Error) -> Self {
        Self::new(err, ErrorSink::default())
    }
}

impl<'a> RootContext for VoidRootContext<'a> {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        self.error_sink
            .observe("failed to create Proxy Wasm Root Context", &self.err);
        false // indicate to Envoy that configuration is not valid
    }
}

impl<'a> Context for VoidRootContext<'a> {}

pub(crate) struct VoidContextSelector {
    err: Error,
}

impl VoidContextSelector {
    pub fn new(err: Error) -> Self {
        VoidContextSelector { err }
    }

    pub fn install(self) {
        let err = Rc::new(self.err);
        proxy_wasm_ext::set_root_context(move |_| {
            // At the moment, `wasm32-unknown-unknown` and `wasm32-wasi` targets
            // do not support stack unwinding.
            // Consequently, in the case of a panic, memory on heap will not be released.
            // Which leaves Envoy no choice but to deem the VM unsafe to use any longer.
            // Even worse, at the moment, Envoy simply crashes whenever a panic happens
            // inside a WebAssembly module.
            // That is why, instead of raising a panic in here, we memorize the error
            // with the intent to report it later in a manner that won't crash Envoy.
            // Specifically, we're relying on the fact that `_start`
            // call will be followed by `proxy_on_vm_start` where we can legally
            // report back to Envoy that VM state is not valid.
            Box::new(VoidVmContext::with_default_ops(Rc::clone(&err)))
        });
    }
}

/// Fake `Proxy Wasm` [`RootContext`] that is used to postpone error handling
/// until a proper moment in the extension lifecycle.
///
/// E.g., if an error occurres inside [`_start`] callback where a WebAssembly module
/// is expected to register all the extensions it provides,
/// we cannot reject invalid Envoy configuration right away - `Envoy` doesn't expect it
/// at this point.
///
/// Instead, we have to memorize the error and wait until [`proxy_on_vm_start`]
/// callback when it will be possible to signal back that extension is not functional.
///
/// [`RootContext`]: https://docs.rs/proxy-wasm/0.1.0/proxy_wasm/traits/trait.RootContext.html
/// [`_start`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#_start
/// [`proxy_on_vm_start`]: https://github.com/proxy-wasm/spec/tree/master/abi-versions/vNEXT#proxy_on_vm_start
struct VoidVmContext<'a> {
    err: Rc<Error>,
    error_sink: &'a dyn ErrorSink,
}

impl<'a> VoidVmContext<'a> {
    fn new(err: Rc<Error>, error_sink: &'a dyn ErrorSink) -> Self {
        VoidVmContext { err, error_sink }
    }

    fn with_default_ops(err: Rc<Error>) -> Self {
        Self::new(err, ErrorSink::default())
    }
}

impl<'a> RootContext for VoidVmContext<'a> {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        self.error_sink
            .observe("failed to initialize WebAssembly module", &self.err);
        false // indicate to Envoy that WebAssembly module is in invalid state
    }
}

impl<'a> Context for VoidVmContext<'a> {}
