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

//! Fake `HTTP Client API`.
//!
//! # Examples
//!
//! #### Basic usage of [`FakeHttpClient`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use std::time::Duration;
//! use envoy::host::HttpClient;
//! use envoy_test::FakeHttpClient;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let mut http_client = FakeHttpClient::default();
//!
//! let request_handle = http_client.send_request(
//!     "example_cluster",
//!     &vec![
//!         (":method", b"GET" as &[u8]),
//!         (":path", b"/stuff"),
//!         (":authority", b"example.org"),
//!     ],
//!     None,
//!     &[],
//!     Duration::from_secs(3),
//! )?;
//!
//! let pending_requests = http_client.drain_pending_requests();
//!
//! assert_eq!(pending_requests.len(), 1);
//! assert_eq!(pending_requests[0].handle, request_handle);
//!
//! # Ok(())
//! # }
//! ```
//!
//! [`FakeHttpClient`]: struct.FakeHttpClient.html

use std::cell::RefCell;
use std::time::Duration;

use envoy::host::http::client::{HttpClient, HttpClientRequestHandle};
use envoy::host::{Bytes, HeaderMap, HeaderName, HeaderValue, Result};

/// Fake `HTTP Client`.
#[derive(Debug, Default)]
pub struct FakeHttpClient {
    counter: RefCell<u32>,
    requests: RefCell<Vec<FakePendingRequest>>,
}

/// Snapshot of an HTTP request made through [`FakeHttpClient`].
///
/// [`FakeHttpClient`]: struct.FakeHttpClient.html
#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FakeHttpClientRequest {
    pub upstream: String,
    pub message: FakeHttpMessage,
    pub timeout: Duration,
}

/// HTTP message.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FakeHttpMessage {
    pub headers: HeaderMap,
    pub body: Bytes,
    pub trailers: HeaderMap,
}

/// Record of a pending HTTP request made through [`FakeHttpClient`].
///
/// [`FakeHttpClient`]: struct.FakeHttpClient.html
#[derive(Debug)]
#[non_exhaustive]
pub struct FakePendingRequest {
    pub request: FakeHttpClientRequest,
    pub handle: HttpClientRequestHandle,
}

/// Builder of a [`FakeHttpClientRequest`].
///
/// [`FakeHttpClientRequest`]: struct.FakeHttpClientRequest.html
#[derive(Debug, Default, Clone)]
pub struct FakeHttpClientRequestBuilder {
    request: FakeHttpClientRequest,
}

/// Snapshot of a response to an HTTP request made through [`FakeHttpClient`].
///
/// [`FakeHttpClient`]: struct.FakeHttpClient.html
#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct FakeHttpClientResponse {
    pub message: FakeHttpMessage,
}

/// Builder of a [`FakeHttpClientResponse`].
///
/// [`FakeHttpClientResponse`]: struct.FakeHttpClientResponse.html
#[derive(Debug, Default, Clone)]
pub struct FakeHttpClientResponseBuilder {
    response: FakeHttpClientResponse,
}

impl HttpClient for FakeHttpClient {
    /// Sends an HTTP request asynchronously.
    fn send_request(
        &self,
        upstream: &str,
        headers: &[(&str, &[u8])],
        body: Option<&[u8]>,
        trailers: &[(&str, &[u8])],
        timeout: Duration,
    ) -> Result<HttpClientRequestHandle> {
        let handle = HttpClientRequestHandle::from(*self.counter.borrow());
        *self.counter.borrow_mut() += 1;
        let request = FakeHttpClientRequest {
            upstream: upstream.to_owned(),
            message: FakeHttpMessage {
                headers: headers.into(),
                body: body.map(|o| o.to_vec()).into(),
                trailers: trailers.into(),
            },
            timeout,
        };
        self.requests
            .borrow_mut()
            .push(FakePendingRequest { request, handle });
        Ok(handle)
    }
}

impl FakeHttpClient {
    /// Returns a list of HTTP requests made since the last call to this method.
    pub fn drain_pending_requests(&self) -> Vec<FakePendingRequest> {
        self.requests.borrow_mut().drain(..).collect()
    }
}

impl FakeHttpClientRequest {
    pub fn builder() -> FakeHttpClientRequestBuilder {
        FakeHttpClientRequestBuilder::default()
    }
}

impl FakeHttpClientRequestBuilder {
    pub fn upstream<U>(mut self, upsteam: U) -> Self
    where
        U: Into<String>,
    {
        self.request.upstream = upsteam.into();
        self
    }

    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let mut headers = self.request.message.headers.into_vec();
        headers.push((name.into(), value.into()));
        self.request.message.headers = headers.into();
        self
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        self.request.message.body = body.into().into();
        self
    }

    pub fn trailer<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let mut trailers = self.request.message.trailers.into_vec();
        trailers.push((name.into(), value.into()));
        self.request.message.trailers = trailers.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.request.timeout = timeout;
        self
    }

    pub fn build(self) -> FakeHttpClientRequest {
        self.request
    }
}

impl FakeHttpClientResponse {
    pub fn builder() -> FakeHttpClientResponseBuilder {
        FakeHttpClientResponseBuilder::default()
    }
}

impl FakeHttpClientResponseBuilder {
    pub fn header<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let mut headers = self.response.message.headers.into_vec();
        headers.push((name.into(), value.into()));
        self.response.message.headers = headers.into();
        self
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        self.response.message.body = body.into().into();
        self
    }

    pub fn trailer<K, V>(mut self, name: K, value: V) -> Self
    where
        K: Into<HeaderName>,
        V: Into<HeaderValue>,
    {
        let mut trailers = self.response.message.trailers.into_vec();
        trailers.push((name.into(), value.into()));
        self.response.message.trailers = trailers.into();
        self
    }

    pub fn build(self) -> FakeHttpClientResponse {
        self.response
    }
}
