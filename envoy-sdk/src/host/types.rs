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

use core::iter::{FromIterator, FusedIterator};
use core::str;

pub use crate::abi::proxy_wasm::types::ByteString;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct HeaderMap {
    entries: Vec<(ByteString, ByteString)>,
}

impl HeaderMap {
    fn from_entries(entries: Vec<(ByteString, ByteString)>) -> Self {
        HeaderMap { entries }
    }

    pub fn builder() -> HeaderMapBuilder {
        HeaderMapBuilder::new()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        HeaderMap {
            entries: Vec::with_capacity(capacity),
        }
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

    /// Returns a reference to the header value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// use envoy::host::HeaderMap;
    ///
    /// let mut headers = HeaderMap::builder()
    ///     .header(":authority", "example.org")
    ///     .build();
    ///
    /// assert_eq!(headers.get(":authority"), Some(&"example.org".into()));
    /// assert_eq!(headers.get(":method"), None);
    /// ```
    pub fn get<Q>(&self, key: Q) -> Option<&ByteString>
    where
        Q: AsRef<[u8]>,
    {
        for i in 0..self.entries.len() {
            if self.entries[i].0 == key.as_ref() {
                return Some(&self.entries[i].1);
            }
        }
        None
    }

    /// Inserts a header.
    ///
    /// If the header has not ben present before, [`None`] is returned.
    /// Otherwise, the value is updated, and the old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// use envoy::host::HeaderMap;
    ///
    /// let mut headers = HeaderMap::builder()
    ///     .header(":authority", "example.org")
    ///     .build();
    ///
    /// assert_eq!(headers.insert(":authority", "example.com"), Some("example.org".into()));
    /// assert_eq!(headers.insert(":method", "GET"), None);
    /// # assert_eq!(headers, HeaderMap::builder().header(":authority", "example.com").header(":method", "GET").build());
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<ByteString>
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.entries.push((key.into(), value.into()));
        for i in 0..self.entries.len() - 1 {
            if self.entries[i].0 == self.entries[self.entries.len() - 1].0 {
                return Some(self.entries.swap_remove(i).1);
            }
        }
        None
    }

    /// Removes a header by name, returning its value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use envoy_sdk as envoy;
    /// use envoy::host::HeaderMap;
    ///
    /// let mut headers = HeaderMap::builder()
    ///     .header(":authority", "example.org")
    ///     .header(":method", "GET")
    ///     .build();
    ///
    /// assert_eq!(headers.remove(":authority"), Some("example.org".into()));
    /// assert_eq!(headers.remove("content-type"), None);
    /// # assert_eq!(headers, HeaderMap::builder().header(":method", "GET").build());
    /// ```
    pub fn remove<Q>(&mut self, key: Q) -> Option<ByteString>
    where
        Q: AsRef<[u8]>,
    {
        for i in 0..self.entries.len() {
            if self.entries[i].0 == key.as_ref() {
                return Some(self.entries.remove(i).1);
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct HeaderMapBuilder {
    map: HeaderMap,
}

impl HeaderMapBuilder {
    pub fn new() -> Self {
        HeaderMapBuilder::default()
    }

    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.map.insert(name, value);
        self
    }

    pub fn build(self) -> HeaderMap {
        self.map
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

impl FromIterator<(ByteString, ByteString)> for HeaderMap {
    fn from_iter<T: IntoIterator<Item = (ByteString, ByteString)>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let mut headers = Self::with_capacity(lower);
        for (name, value) in iterator {
            headers.insert(name, value);
        }
        headers
    }
}

impl<'a> FromIterator<&'a (&'a str, &'a str)> for HeaderMap {
    fn from_iter<T: IntoIterator<Item = &'a (&'a str, &'a str)>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let mut headers = Self::with_capacity(lower);
        for (name, value) in iterator {
            headers.insert(*name, *value);
        }
        headers
    }
}

impl From<&HeaderMap> for HeaderMap {
    fn from(other: &HeaderMap) -> Self {
        other.clone()
    }
}

impl From<Vec<(ByteString, ByteString)>> for HeaderMap {
    fn from(entries: Vec<(ByteString, ByteString)>) -> Self {
        Self::from_entries(entries)
    }
}

impl From<&[(&str, &str)]> for HeaderMap {
    fn from(entries: &[(&str, &str)]) -> Self {
        Self::from_entries(
            entries
                .iter()
                .map(|(name, value)| ((*name).into(), (*value).to_owned().into()))
                .collect(),
        )
    }
}
