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

use core::iter::FusedIterator;
use core::str;

pub use crate::abi::proxy_wasm::types::ByteString;

#[derive(Debug, Default)]
pub struct HeaderMap {
    entries: Vec<(ByteString, ByteString)>,
}

impl HeaderMap {
    fn from_entries(entries: Vec<(ByteString, ByteString)>) -> Self {
        HeaderMap { entries }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn as_slice(&self) -> &[(ByteString, ByteString)] {
        self.entries.as_slice()
    }

    pub fn into_vec(self) -> Vec<(ByteString, ByteString)> {
        self.entries
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = (&'a ByteString, &'a ByteString);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    inner: std::slice::Iter<'a, (ByteString, ByteString)>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a ByteString, &'a ByteString);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|&(ref name, ref value)| (name, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl IntoIterator for HeaderMap {
    type Item = (ByteString, ByteString);
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

pub struct IntoIter {
    inner: std::vec::IntoIter<(ByteString, ByteString)>,
}

impl Iterator for IntoIter {
    type Item = (ByteString, ByteString);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl FusedIterator for IntoIter {}

impl From<Vec<(ByteString, ByteString)>> for HeaderMap {
    fn from(entries: Vec<(ByteString, ByteString)>) -> Self {
        Self::from_entries(entries)
    }
}

impl From<&[(&str, &[u8])]> for HeaderMap {
    fn from(entries: &[(&str, &[u8])]) -> Self {
        Self::from_entries(
            entries
                .iter()
                .map(|(name, value)| ((*name).into(), (*value).to_owned().into()))
                .collect(),
        )
    }
}
