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
use proxy_wasm::traits::RootContext;
use proxy_wasm::types::LogLevel;

use envoy::extension;
use envoy::on_module_load;

use access_logger::SampleAccessLogger;

// Generate a `_start` function with a given code that will be called by Envoy
// to let WebAssembly module initialize itself.
on_module_load! { initialize(); }

/// Does one-time initialization.
fn initialize() {
    proxy_wasm::set_log_level(LogLevel::Info);

    // Register Access logger extension
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        // Inject dependencies on Envoy host APIs
        let logger =
            SampleAccessLogger::with_default_ops().expect("unable to initialize extension");

        // Bridge between Access logger abstraction and Envoy ABI
        Box::new(extension::access_logger::LoggerContext::with_default_ops(
            logger,
        ))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize() {
        initialize()
    }
}
