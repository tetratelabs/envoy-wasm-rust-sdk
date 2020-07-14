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

//! Errors specific to extension callback methods.

use std::fmt;

/// An error while WebAssembly module is being initialized.
#[derive(Debug)]
pub(crate) enum ModuleError {
    DuplicateRegistration(String),
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ModuleError::*;
        match &self {
            DuplicateRegistration(name) => write!(
                f,
                "WebAssembly module attempted to register 2 different extensions under the same `root_id` \"{}\"",
                name,
            ),
        }
    }
}

impl std::error::Error for ModuleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// An error in the configuration of Envoy extension.
#[derive(Debug)]
pub(crate) enum ConfigurationError {
    UnknownExtension {
        requested: String,
        available: Vec<String>,
    },
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConfigurationError::*;
        match &self {
            UnknownExtension { requested, available } => write!(
                f,
                "WebAssembly module has no extension with `root_id` \"{}\"; valid `root_id` values are: {:?}",
                requested, available
            ),
        }
    }
}

impl std::error::Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
