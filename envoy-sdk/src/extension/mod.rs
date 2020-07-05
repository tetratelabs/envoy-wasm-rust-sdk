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

extern crate std;

use std::fmt;
use std::prelude::v1::*;

pub use factory::Factory;

pub mod access_logger;
pub mod factory;
pub mod filter;

#[derive(Debug)]
pub enum Error {
    HostCall(&'static str, proxy_wasm::types::Status),
    Extension,
}

impl From<(&'static str, proxy_wasm::types::Status)> for Error {
    fn from(pair: (&'static str, proxy_wasm::types::Status)) -> Self {
        Error::HostCall(pair.0, pair.1)
    }
}

/// The type returned by extension methods.
pub type Result<T> = core::result::Result<T, Error>;

/// Opaque identifier of an extension instance.
#[derive(Debug, PartialEq, Eq)]
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
