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

//! The `Error` type and helpers.
//!
//! # Examples
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::error::{bail, Result};
//!
//! fn on_request() -> Result<()> {
//!     // ...
//!     bail!("unexpected state");
//! }
//! ```
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::error::{ensure, Result};
//!
//! fn on_request() -> Result<()> {
//! #   let body_len = 0;
//!     ensure!(body_len != 0, "request body must not be empty");
//!     // ...
//! #   Ok(())
//! }
//! ```
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::error::{format_err, Result};
//!
//! fn on_request() -> Result<()> {
//! #   let method = "";
//!     if method == "DELETE" {
//!         return Err(format_err!("{} method is not allowed", method));
//!     }
//!     // ...
//! #   Ok(())
//! }
//! ```
//!
//! ```
//! # use envoy_sdk as envoy;
//! use envoy::error::{ErrorContext, Result};
//! use envoy::host::Clock;
//!
//! fn on_request() -> Result<()> {
//! #   let instance_id = 123;
//!     let now = Clock::default().now()
//!         .with_context(|| format!("Failed to get time of the request {}", instance_id))?;
//!     // ...
//! #   Ok(())
//! }
//! ```

pub use anyhow::{bail, ensure, format_err};
pub use anyhow::{Context as ErrorContext, Error, Result};
