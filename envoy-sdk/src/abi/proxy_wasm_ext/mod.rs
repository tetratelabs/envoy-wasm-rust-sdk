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

//! Extensions to [`proxy-wasm`] SDK.
//!
//! The rest of the code in this crate should use this module instead of
//! the original [`proxy-wasm`].
//!
//! [`proxy-wasm`]: https://docs.rs/proxy-wasm/

pub use proxy_wasm::{set_log_level, set_root_context, traits};

pub mod hostcalls;
pub mod types;
