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

use proxy_wasm::traits::RootContext;
use proxy_wasm::types::LogLevel;

use envoy_sdk::extension;
use envoy_sdk::start;

use access_logger::SampleAccessLogger;

// Generate a `_start` function that is called by Envoy
// when a new instance of WebAssembly module is created.
start! { on_module_start(); }

/// Does one-time initialization.
fn on_module_start() {
    proxy_wasm::set_log_level(LogLevel::Info);

    // Register Access logger extension
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        // Inject dependencies on Envoy host APIs
        let logger = SampleAccessLogger::with_default_ops();

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
    fn should_start() {
        on_module_start()
    }
}
