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

//! `Envoy` `Host APIs` provided for use by extensions.
//!
//! # Structure
//!
//! Every supported `Envoy` `Host API` is represented by a trait object,
//! e.g. [`Clock`], [`HttpClient`], [`Stats`], etc.
//!
//! Extensions get parameterized with a concrete implementation of `Host API`s
//! at the time of their construction.
//!
//! This way, you can swap a real `Host API` with a mock when unit testing your
//! extension.
//!
//! # Examples
//!
//! #### Parameterize HTTP Filter extension with `Envoy` [`Clock`]:
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::host::Clock;
//!
//! struct MyHttpFilter<'a> {
//!     clock: &'a dyn Clock,
//! }
//!
//! impl<'a> MyHttpFilter<'a> {
//!     /// Creates a new instance parameterized with a given [`Clock`] implementation.
//!     pub fn new(clock: &'a dyn Clock) -> Self {
//!         MyHttpFilter { clock }
//!     }
//!
//!     /// Creates a new instance parameterized with the default [`Clock`] implementation.
//!     pub fn default() -> Self {
//!         Self::new(Clock::default())
//!     }
//! }
//! ```
//!
//! [`Clock`]: time/trait.Clock.html
//! [`HttpClient`]: http/client/trait.HttpClient.html
//! [`Stats`]: stats/trait.Stats.html

pub(crate) use self::error::function;

pub use self::error::{Error, Result};
pub use self::http::client::HttpClient;
pub use self::shared_data::SharedData;
pub use self::shared_queue::SharedQueue;
pub use self::stats::Stats;
pub use self::stream_info::StreamInfo;
pub use self::time::Clock;
pub use self::types::{Bytes, HeaderMap, HeaderName, HeaderValue};

mod types;

pub mod buffer;
pub mod error;
pub mod http;
pub mod log;
pub mod shared_data;
pub mod shared_queue;
pub mod stats;
pub mod stream_info;
pub mod time;
