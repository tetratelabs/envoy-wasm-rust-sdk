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

//! Fake `HTTP API`.

use envoy::host::{ByteString, HeaderMap};

pub mod client;

/// HTTP message.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FakeHttpMessage {
    pub headers: HeaderMap,
    pub body: ByteString,
    pub trailers: HeaderMap,
}

impl FakeHttpMessage {
    pub fn builder() -> FakeHttpMessageBuilder {
        FakeHttpMessageBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct FakeHttpMessageBuilder {
    message: FakeHttpMessage,
}

impl FakeHttpMessageBuilder {
    pub fn new() -> Self {
        FakeHttpMessageBuilder::default()
    }

    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.message.headers.insert(name, value);
        self
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<ByteString>,
    {
        self.message.body = body.into();
        self
    }

    pub fn trailer<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<ByteString>,
        V: Into<ByteString>,
    {
        self.message.trailers.insert(name, value);
        self
    }

    pub fn build(self) -> FakeHttpMessage {
        self.message
    }
}
