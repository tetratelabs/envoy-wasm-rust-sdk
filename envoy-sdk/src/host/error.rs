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

//! Errors specific to `Envoy` host APIs.

use std::fmt;

use crate::abi::proxy_wasm_ext::types::Status;

pub(crate) fn function(module: &'static str, function: &'static str) -> Function {
    Function::new(module, function)
}

/// Represents a host ABI function.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Function {
    module: &'static str,
    function: &'static str,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.function)
    }
}

impl Function {
    pub(crate) fn new(module: &'static str, function: &'static str) -> Self {
        Function { module, function }
    }

    pub(crate) fn call_error(&self, status: Status) -> Error {
        Error::new(self, status)
    }
}

/// The error type for host API functions.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Error {
    function: Function,
    status: Status,
}

impl Error {
    pub(crate) fn new(function: &Function, status: Status) -> Self {
        Error {
            function: function.clone(),
            status,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "call to host ABI function '{}' failed with status code {}",
            self.function, self.status as u32
        )
    }
}

impl std::error::Error for Error {}

/// A specialized [`Result`] type for use in host API functions.
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = core::result::Result<T, Error>;
