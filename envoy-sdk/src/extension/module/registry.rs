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

use crate::abi::proxy_wasm_ext;
use crate::abi::proxy_wasm_ext::traits::{ChildContext, RootContext};
use crate::extension::{access_logger, factory, filter::http, filter::network};
use crate::extension::{InstanceId, Result};

type NewRootContextFn = dyn FnMut(u32) -> Box<dyn RootContext>;

/// Registry of extensions provided by the WebAssembly module.
pub struct Registry {
    access_logger: Option<Box<NewRootContextFn>>,
    network_filter: Option<Box<NewRootContextFn>>,
    http_filter: Option<Box<NewRootContextFn>>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            access_logger: None,
            network_filter: None,
            http_filter: None,
        }
    }

    pub fn add_access_logger<T, F>(mut self, mut new: F) -> Self
    where
        T: access_logger::Logger + 'static,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        self.access_logger = Some(Box::new(move |context_id| {
            let logger =
                new(InstanceId::from(context_id)).expect("unable to initialize Access Logger");

            // Bridge between Access Logger abstraction and Envoy ABI
            Box::new(access_logger::LoggerContext::with_default_ops(logger))
        }));
        self
    }

    pub fn add_network_filter<T, F>(mut self, mut new: F) -> Self
    where
        T: factory::Factory + 'static,
        T::Extension: network::Filter,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        self.network_filter = Some(Box::new(move |context_id| {
            let network_filter_factory = new(InstanceId::from(context_id))
                .expect("unable to initialize Network Filter factory");

            // Bridge between Network Filter Factory abstraction and Envoy ABI
            Box::new(factory::FactoryContext::with_default_ops(
                network_filter_factory,
                |network_filter_factory, instance_id| -> ChildContext {
                    let network_filter =
                        <_ as factory::Factory>::new_extension(network_filter_factory, instance_id)
                            .unwrap();

                    // Bridge between Network Filter abstraction and Envoy ABI
                    ChildContext::StreamContext(Box::new(network::FilterContext::with_default_ops(
                        network_filter,
                    )))
                },
            ))
        }));
        self
    }

    pub fn add_http_filter<T, F>(mut self, mut new: F) -> Self
    where
        T: factory::Factory + 'static,
        T::Extension: http::Filter,
        F: FnMut(InstanceId) -> Result<T> + 'static,
    {
        self.http_filter = Some(Box::new(move |context_id| {
            let http_filter_factory = new(InstanceId::from(context_id))
                .expect("unable to initialize HTTP Filter factory");

            // Bridge between HTTP Filter Factory abstraction and Envoy ABI
            Box::new(factory::FactoryContext::with_default_ops(
                http_filter_factory,
                |http_filter_factory, instance_id| -> ChildContext {
                    let http_filter =
                        <_ as factory::Factory>::new_extension(http_filter_factory, instance_id)
                            .unwrap();

                    // Bridge between HTTP Filter abstraction and Envoy ABI
                    ChildContext::HttpContext(Box::new(http::FilterContext::with_default_ops(
                        http_filter,
                    )))
                },
            ))
        }));
        self
    }

    pub(super) fn install(self) -> Result<()> {
        if let Some(access_logger) = self.access_logger {
            proxy_wasm_ext::set_root_context(access_logger);
        }
        if let Some(network_filter) = self.network_filter {
            proxy_wasm_ext::set_root_context(network_filter);
        }
        if let Some(http_filter) = self.http_filter {
            proxy_wasm_ext::set_root_context(http_filter);
        }
        Ok(())
    }
}
