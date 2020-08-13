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

//! `Envoy` `Log API`.

pub use crate::abi::proxy_wasm_ext::types::LogLevel;

#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};

/// Sets the global maximum log level.
///
/// # Examples
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::host::log::{self, LogLevel};
///
/// // change max log level (by default, `LogLevel::Info`)
/// log::set_max_level(LogLevel::Debug);
/// ```
#[cfg(feature = "log")]
pub use crate::abi::proxy_wasm_ext::set_log_level as set_max_level;
