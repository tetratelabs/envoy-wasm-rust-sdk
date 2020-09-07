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

//! `Unit Test Framework` accompanying `Rust` SDK for WebAssembly-based `Envoy` extensions.
//!
//! ## Supported "fakes"
//!
//! * [`FakeClock`]
//! * [`FakeHttpClient`]
//! * [`FakeStats`]
//! * [`FakeStreamInfo`]
//!
//! [`FakeClock`]: host/time/index.html
//! [`FakeHttpClient`]: host/http/client/index.html
//! [`FakeStats`]: host/stats/index.html
//! [`FakeStreamInfo`]: host/stream_info/index.html

#![doc(html_root_url = "https://docs.rs/envoy-sdk-test/0.0.1")]

pub use self::host::*;

pub mod host;
