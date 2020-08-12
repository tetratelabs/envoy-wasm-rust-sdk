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

//! Errors specific to interaction with `Envoy` host ABI.

use std::fmt;

use crate::abi::proxy_wasm::types::Status;

pub use crate::common::{Error, Result};

/// An error returned from the call to Envoy ABI.
#[derive(Debug)]
pub(crate) struct HostCallError {
    function: Function,
    status: Status,
}

impl HostCallError {
    fn new(function: Function, status: Status) -> Self {
        HostCallError { function, status }
    }
}

impl fmt::Display for HostCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "call to the host ABI function \"{}\" has failed with the status code {}",
            self.function, self.status as u32
        )
    }
}

impl std::error::Error for HostCallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// An error at parsing a return value from a call to Envoy ABI.
#[derive(Debug)]
pub(crate) struct HostResponseError {
    function: Function,
    err: Error,
}

impl HostResponseError {
    fn new(function: Function, err: Error) -> Self {
        HostResponseError { function, err }
    }
}

impl fmt::Display for HostResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "failed to parse value returned by the host ABI function \"{}\": {}",
            self.function, self.err,
        )
    }
}

impl std::error::Error for HostResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.err)
    }
}

/// Represents a host ABI function.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub(crate) struct Function {
    module: &'static str,
    function: &'static str,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.function)
    }
}

impl Function {
    fn new(module: &'static str, function: &'static str) -> Self {
        Function { module, function }
    }

    pub fn into_call_error(self, status: Status) -> HostCallError {
        HostCallError::new(self, status)
    }

    pub fn into_parse_error(self, err: Error) -> HostResponseError {
        HostResponseError::new(self, err)
    }
}

pub(crate) fn function(module: &'static str, function: &'static str) -> Function {
    Function::new(module, function)
}
