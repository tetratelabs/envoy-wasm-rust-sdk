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

use envoy::proxy_wasm;
use proxy_wasm::traits::{ChildContext, RootContext};
use proxy_wasm::types::LogLevel;

use envoy::extension;
use envoy::extension::factory;
use envoy::extension::filter::http;
use envoy::start;

use http_filter::SampleHttpFilterFactory;

// Generate a `_start` function that is called by Envoy
// when a new instance of WebAssembly module is created.
start! { on_module_start(); }

/// Does one-time initialization.
fn on_module_start() {
    proxy_wasm::set_log_level(LogLevel::Info);

    // Register HTTP filter extension
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        // Inject dependencies on Envoy host APIs
        let http_filter_factory =
            SampleHttpFilterFactory::with_default_ops().expect("unable to initialize extension");

        // Bridge between HTTP filter factory abstraction and Envoy ABI
        Box::new(factory::FactoryContext::with_default_ops(
            http_filter_factory,
            |http_filter_factory, instance_id| -> ChildContext {
                let http_filter = <_ as extension::factory::Factory>::new_extension(
                    http_filter_factory,
                    instance_id,
                )
                .unwrap();

                // Bridge between HTTP filter abstraction and Envoy ABI
                ChildContext::HttpContext(Box::new(http::FilterContext::with_default_ops(
                    http_filter,
                )))
            },
        ))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_start() {
        on_module_start()
    }
}
