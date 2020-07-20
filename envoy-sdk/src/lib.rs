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

//! `Rust` SDK for WebAssembly-based `Envoy` extensions.
//!
//! ## TLDR
//!
//! ```
//! # use envoy_sdk as envoy;
//! # use envoy::extension::{HttpFilter, Result};
//! # use envoy::extension::filter::http::{FilterHeadersStatus, RequestHeadersOps};
//! # use envoy::host::log;
//! #
//! /// My very own `HttpFilter`.
//! struct MyHttpFilter;
//!
//! impl HttpFilter for MyHttpFilter {
//!     fn on_request_headers(&mut self, _num_headers: usize, _ops: &dyn RequestHeadersOps) -> Result<FilterHeadersStatus> {
//!         log::info!("proxying an HTTP request");
//!         Ok(FilterHeadersStatus::Continue)
//!     }
//! }
//! ```
//!
//! ## Supported Extension Types
//!
//! `Envoy SDK` can help you to develop the following types of extensions:
//! * [`HttpFilter`]
//! * [`NetworkFilter`]
//! * [`AccessLogger`]
//!
//! ## Supported Envoy APIs
//!
//! You can use the following `Envoy APIs` in your extensions:
//! * [`Clock`]
//! * [`HttpClient`]
//! * [`Log`]
//! * [`Stats`]
//! * [`StreamInfo`]
//! * [`SharedData`]
//! * [`SharedQueue`]
//!
//! ## Example extensions
//!
//! * [`Sample HTTP Filter`][`SampleHttpFilter`]
//! * [`Sample Network Filter`][`SampleNetworkFilter`]
//! * [`Sample Access Logger`][`SampleAccessLogger`]
//!
//! ## How To
//!
//! * [How To make my extension configurable?][`HowToConfigure`]
//! * [How To share stats between filter instances?][`HowToShareStats`]
//! * [How To use HttpClient?][`HowToUseHttpClient`]
//!
//! [`HttpFilter`]: extension/filter/http/index.html
//! [`NetworkFilter`]: extension/filter/network/index.html
//! [`AccessLogger`]: extension/access_logger/index.html
//!
//! [`Clock`]: host/time/trait.Clock.html
//! [`HttpClient`]: host/http/client/trait.HttpClient.html
//! [`Log`]: host/log/index.html
//! [`Stats`]: host/stats/trait.Stats.html
//! [`StreamInfo`]: host/stream_info/trait.StreamInfo.html
//! [`SharedData`]: host/shared_data/trait.SharedData.html
//! [`SharedQueue`]: host/shared_queue/trait.SharedQueue.html
//!
//! [`SampleHttpFilter`]: https://github.com/tetratelabs/envoy-wasm-rust-sdk/tree/master/examples/http-filter
//! [`SampleNetworkFilter`]: https://github.com/tetratelabs/envoy-wasm-rust-sdk/tree/master/examples/network-filter
//! [`SampleAccessLogger`]: https://github.com/tetratelabs/envoy-wasm-rust-sdk/tree/master/examples/access-logger
//!
//! [`HowToConfigure`]: extension/factory/trait.ExtensionFactory.html#examples
//! [`HowToShareStats`]: extension/factory/trait.ExtensionFactory.html#examples
//! [`HowToUseHttpClient`]: host/http/client/trait.HttpClient.html#examples

#![doc(html_root_url = "https://docs.rs/envoy-sdk/0.0.4")]

mod abi;

pub mod error;
pub mod extension;
pub mod host;
