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

use envoy::extension::{on_module_load, Registry, Result};

use network_filter::SampleNetworkFilterFactory;

// Generate the `_start` function that will be called by `Envoy` to let
// WebAssembly module initialize itself.
on_module_load! { initialize }

/// Does one-time initialization.
///
/// Returns a registry of extensions provided by this module.
fn initialize() -> Result<Registry> {
    Registry::new().add_network_filter(|_instance_id| SampleNetworkFilterFactory::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize() {
        assert!(initialize().is_ok());
    }
}
