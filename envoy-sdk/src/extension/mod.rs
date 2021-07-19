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

//! `Envoy` `Extension APIs` to be implemented by extensions.

use std::fmt;

pub use self::error::{Error, ErrorContext, Result};
pub use self::factory::{ConfigStatus, DrainStatus, ExtensionFactory};
pub use self::filter::http::HttpFilter;
pub use self::module::{install, Module};
pub use crate::entrypoint;

mod module;

pub mod error;
pub mod factory;
pub mod filter;

/// Opaque identifier of an extension instance.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InstanceId(u32);

impl From<u32> for InstanceId {
    fn from(context_id: u32) -> Self {
        InstanceId(context_id)
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
