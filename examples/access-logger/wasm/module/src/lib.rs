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
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

use access_logger::SampleAccessLogger;

start! { on_module_start(); }

/// Is called when a new instance of WebAssembly module is created.
fn on_module_start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        // Inject dependencies on Envoy host APIs
        let logger = SampleAccessLogger::new(&time::ops::Host, &clients::http::ops::Host);
        Box::new(extension::access_logger::LoggerContext::new(
            logger,
            &extension::access_logger::ops::Host,
            &clients::http::ops::Host,
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
