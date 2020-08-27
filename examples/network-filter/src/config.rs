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

use std::convert::TryFrom;

use serde::Deserialize;

use envoy::extension;

/// Configuration for a Sample Network Filter.
#[derive(Deserialize, Debug)]
pub struct SampleNetworkFilterConfig {
    #[serde(default)]
    pub param: String,
}

impl TryFrom<&[u8]> for SampleNetworkFilterConfig {
    type Error = extension::Error;

    /// Parses filter configuration from JSON.
    fn try_from(value: &[u8]) -> extension::Result<Self> {
        serde_json::from_slice(value).map_err(extension::Error::from)
    }
}

impl Default for SampleNetworkFilterConfig {
    /// Creates the default configuration.
    fn default() -> Self {
        SampleNetworkFilterConfig {
            param: String::default(),
        }
    }
}
