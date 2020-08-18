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

//! Interface for mutating payload data proxied by `Envoy`.

pub(crate) use self::internal::TransformExecutor;

/// Represents a transformation on data proxied by `Envoy`, i.e. on
/// `TCP`-level payload data or on `HTTP`-level request/response body data.
#[derive(Debug)]
pub struct Transform<'a> {
    inner: TransformKind<'a>,
}

/// List of transformations supported by `Proxy Wasm` inside `Envoy`.
#[derive(Debug)]
pub(crate) enum TransformKind<'a> {
    /// Insert given data into the beginning of a buffer.
    Prepend(&'a [u8]),
    /// Insert given data into the end of a buffer.
    Append(&'a [u8]),
    /// Replace contents of a buffer with given data.
    Replace(&'a [u8]),
}

impl<'a> Transform<'a> {
    fn new(kind: TransformKind<'a>) -> Self {
        Transform { inner: kind }
    }

    /// Returns a transformation that will insert given data into the beginning of a buffer.
    pub fn prepend<T>(data: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self::new(TransformKind::Prepend(data.as_ref()))
    }

    /// Returns a transformation that will insert given data into the end of a buffer.
    pub fn append<T>(data: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self::new(TransformKind::Append(data.as_ref()))
    }

    /// Returns a transformation that will replace contents of a buffer with given data.
    pub fn replace_with<T>(data: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self::new(TransformKind::Replace(data.as_ref()))
    }
}

#[doc(hidden)]
pub mod internal {
    use super::*;
    use crate::host;

    /// Applies a transformation to data proxied by `Envoy`.
    pub trait TransformExecutor {
        fn execute<F>(self, mutate: F) -> host::Result<()>
        where
            F: FnOnce(usize, usize, &[u8]) -> host::Result<()>;
    }

    impl<'a> TransformExecutor for Transform<'a> {
        /// Executes transformation in terms of primitives supported by `Proxy Wasm`.
        fn execute<F>(self, mutate: F) -> host::Result<()>
        where
            F: FnOnce(usize, usize, &[u8]) -> host::Result<()>,
        {
            // implementation based on `envoyproxy/envoy-wasm`
            use TransformKind::*;
            match self.inner {
                Prepend(data) => mutate(0, 0, data),
                Replace(data) => mutate(0, usize::MAX, data),
                Append(data) => mutate(usize::MAX, usize::MAX, data),
            }
        }
    }
}
