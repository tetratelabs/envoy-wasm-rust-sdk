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
use std::{fmt, ops};

pub use crate::abi::proxy_wasm::types::HeaderValue;

#[derive(Debug, Default)]
pub struct Bytes {
    data: Vec<u8>,
}

impl Bytes {
    pub fn new(data: Vec<u8>) -> Self {
        Bytes { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<Option<Vec<u8>>> for Bytes {
    fn from(buffer: Option<Vec<u8>>) -> Self {
        match buffer {
            Some(data) => Self::new(data),
            None => Self::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct HeaderMap {
    entries: Vec<(HeaderName, HeaderValue)>,
}

impl HeaderMap {
    fn new(entries: Vec<(HeaderName, HeaderValue)>) -> Self {
        HeaderMap { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn as_slice(&self) -> &[(HeaderName, HeaderValue)] {
        self.entries.as_slice()
    }

    pub fn into_vec(self) -> Vec<(HeaderName, HeaderValue)> {
        self.entries
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.entries.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = (&'a HeaderName, &'a HeaderValue);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    inner: std::slice::Iter<'a, (HeaderName, HeaderValue)>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a HeaderName, &'a HeaderValue);

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
    type Item = (HeaderName, HeaderValue);
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            inner: self.entries.into_iter(),
        }
    }
}

pub struct IntoIter {
    inner: std::vec::IntoIter<(HeaderName, HeaderValue)>,
}

impl Iterator for IntoIter {
    type Item = (HeaderName, HeaderValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl FusedIterator for IntoIter {}

impl From<Vec<(String, HeaderValue)>> for HeaderMap {
    fn from(entries: Vec<(String, HeaderValue)>) -> Self {
        Self::new(
            entries
                .into_iter()
                .map(|(name, value)| ((*name).into(), value))
                .collect(),
        )
    }
}

impl From<&[(&str, &[u8])]> for HeaderMap {
    fn from(entries: &[(&str, &[u8])]) -> Self {
        Self::new(
            entries
                .iter()
                .map(|(name, value)| ((*name).into(), (*value).to_owned().into()))
                .collect(),
        )
    }
}

#[derive(Eq, PartialEq)]
pub struct HeaderName {
    inner: String,
}

impl HeaderName {
    /// Returns a `str` representation of the header.
    ///
    /// The returned string will always be lower case.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_ref()
    }
}

impl From<String> for HeaderName {
    fn from(mut data: String) -> Self {
        data.make_ascii_lowercase();
        Self { inner: data }
    }
}

impl From<&str> for HeaderName {
    fn from(data: &str) -> Self {
        data.to_owned().into()
    }
}

impl AsRef<str> for HeaderName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ops::Deref for HeaderName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Debug for HeaderName {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), fmt)
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}
