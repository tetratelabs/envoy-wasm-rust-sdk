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

//! ABI between Envoy and Wasm extensions.
//!
//! At the moment, [`proxy-wasm`] is the only such ABI.
//!
//! In the long term, we anticipate `Envoy` will also get feature-specific ABIs,
//! e.g. one for HTTP Tracers, another for custom Clusters, etc.
//!
//! [`proxy-wasm`]: https://github.com/proxy-wasm/spec

pub mod proxy_wasm_ext;
