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

use super::{ContextFactory, ContextFactoryHashMap};

use crate::abi::proxy_wasm::traits::{ChildContext, HttpContext, RootContext, StreamContext};
use crate::extension::access_logger::{AccessLogger, AccessLoggerContext};
use crate::extension::error::ModuleError;
use crate::extension::factory::{ExtensionFactory, ExtensionFactoryContext};
use crate::extension::filter::http::{HttpFilter, HttpFilterContext, VoidHttpFilterContext};
use crate::extension::filter::network::{
    NetworkFilter, NetworkFilterContext, VoidNetworkFilterContext,
};
use crate::extension::{InstanceId, Result};

/// Registry of extensions provided by the WebAssembly module.
pub struct Registry {
    factories: ContextFactoryHashMap,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            factories: ContextFactoryHashMap::new(),
        }
    }

    fn add_extension(mut self, name: &'static str, factory: Box<ContextFactory>) -> Result<Self> {
        if self.factories.insert(name.to_string(), factory).is_some() {
            Err(ModuleError::DuplicateRegistration(name.to_string()).into())
        } else {
            Ok(self)
        }
    }

    pub fn add_access_logger<T, F>(self, mut new: F) -> Result<Self>
    where
        T: AccessLogger + 'static,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        let factory = Box::new(move |context_id| -> Result<Box<dyn RootContext>> {
            let logger = new(InstanceId::from(context_id))?;

            // Bridge between Access Logger abstraction and Proxy Wasm ABI
            Ok(Box::new(AccessLoggerContext::with_default_ops(logger)))
        });
        self.add_extension(T::NAME, factory)
    }

    pub fn add_network_filter<T, F>(self, mut new: F) -> Result<Self>
    where
        T: ExtensionFactory + 'static,
        T::Extension: NetworkFilter,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        let factory = Box::new(move |context_id| -> Result<Box<dyn RootContext>> {
            let network_filter_factory = new(InstanceId::from(context_id))?;

            // Bridge between Network Filter Factory abstraction and Proxy Wasm ABI
            Ok(Box::new(ExtensionFactoryContext::with_default_ops(
                network_filter_factory,
                |network_filter_factory, instance_id| -> ChildContext {
                    let stream_context: Box<dyn StreamContext> =
                        match <T as ExtensionFactory>::new_extension(
                            network_filter_factory,
                            instance_id,
                        ) {
                            Ok(network_filter) => {
                                Box::new(NetworkFilterContext::with_default_ops(network_filter))
                            }
                            Err(err) => Box::new(VoidNetworkFilterContext::with_default_ops(err)),
                        };
                    // Bridge between Network Filter abstraction and Proxy Wasm ABI
                    ChildContext::StreamContext(stream_context)
                },
            )))
        });
        self.add_extension(T::NAME, factory)
    }

    pub fn add_http_filter<T, F>(self, mut new: F) -> Result<Self>
    where
        T: ExtensionFactory + 'static,
        T::Extension: HttpFilter,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        let factory = Box::new(move |context_id| -> Result<Box<dyn RootContext>> {
            let http_filter_factory = new(InstanceId::from(context_id))?;

            // Bridge between HTTP Filter Factory abstraction and Proxy Wasm ABI
            Ok(Box::new(ExtensionFactoryContext::with_default_ops(
                http_filter_factory,
                |http_filter_factory, instance_id| -> ChildContext {
                    let http_context: Box<dyn HttpContext> =
                        match <T as ExtensionFactory>::new_extension(
                            http_filter_factory,
                            instance_id,
                        ) {
                            Ok(http_filter) => {
                                Box::new(HttpFilterContext::with_default_ops(http_filter))
                            }
                            Err(err) => Box::new(VoidHttpFilterContext::with_default_ops(err)),
                        };
                    // Bridge between HTTP Filter abstraction and Proxy Wasm ABI
                    ChildContext::HttpContext(http_context)
                },
            )))
        });
        self.add_extension(T::NAME, factory)
    }
}

impl Into<ContextFactoryHashMap> for Registry {
    fn into(self) -> ContextFactoryHashMap {
        self.factories
    }
}
