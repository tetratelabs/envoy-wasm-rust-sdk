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

//! Interface for mutating data buffered in `Envoy`.

use crate::host;

pub struct BufferAction<'a> {
    inner: BufferActionKind<'a>,
}

impl<'a> BufferAction<'a> {
    pub fn prepend(data: &'a [u8]) -> BufferAction<'a> {
        BufferAction {
            inner: BufferActionKind::Prepend(data),
        }
    }

    pub fn append(data: &'a [u8]) -> BufferAction<'a> {
        BufferAction {
            inner: BufferActionKind::Append(data),
        }
    }

    pub fn replace_with(data: &'a [u8]) -> BufferAction<'a> {
        BufferAction {
            inner: BufferActionKind::Replace(data),
        }
    }

    #[doc(hidden)]
    pub fn execute<F>(self, mutate: F) -> host::Result<()>
    where
        F: FnOnce(usize, usize, &[u8]) -> host::Result<()>,
    {
        // implementation based on `envoyproxy/envoy-wasm`
        use BufferActionKind::*;
        match self.inner {
            Prepend(data) => mutate(0, 0, data),
            Replace(data) => mutate(0, usize::MAX, data),
            Append(data) => mutate(usize::MAX, usize::MAX, data),
        }
    }
}

/// List of mutations supported by `Proxy Wasm` on `Envoy` side.
pub(crate) enum BufferActionKind<'a> {
    Prepend(&'a [u8]),
    Append(&'a [u8]),
    Replace(&'a [u8]),
}
