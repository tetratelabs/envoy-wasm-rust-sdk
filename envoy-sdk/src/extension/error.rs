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

use crate::host;

/// The error type for extension callback methods.
#[derive(Debug)]
pub enum Error {
    HostCall(host::Error),
    Extension(Box<dyn std::error::Error>),
}

impl Error {
    pub fn new<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error>>,
    {
        Error::Extension(error.into())
    }
}

impl From<host::Error> for Error {
    fn from(err: host::Error) -> Self {
        Error::HostCall(err)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Extension(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            HostCall(ref e) => e.fmt(f),
            Extension(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            HostCall(ref e) => Some(e),
            Extension(ref e) => Some(e.as_ref()),
        }
    }
}

/// A specialized [`Result`] type for use in extension callback methods.
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = core::result::Result<T, Error>;
