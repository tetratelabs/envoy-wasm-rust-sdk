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

use std::time::Duration;

use envoy::host::http::client::HttpClient;
use envoy::host::{HeaderMap, Result};

use envoy_sdk_test as envoy_test;
use envoy_test::host::FakeHttpClient;

#[test]
fn test_fake_http_client() -> Result<()> {
    let mut http_client = FakeHttpClient::default();

    let request_handle = http_client.send_request(
        "example_cluster",
        &[
            (":method", b"GET"),
            (":path", b"/stuff"),
            (":authority", b"example.org"),
        ],
        Some(b"example body"),
        &[("grpc-status", b"0"), ("grpc-message", b"OK")],
        Duration::from_secs(3),
    )?;

    let pending_requests = http_client.drain_pending_requests();

    assert_eq!(pending_requests.len(), 1);

    let pending = &pending_requests[0];

    assert_eq!(pending.handle, request_handle);
    assert_eq!(pending.request.upstream, "example_cluster");
    assert_eq!(
        pending.request.headers.as_slice(),
        HeaderMap::from(
            [
                (":method", "GET".as_bytes()),
                (":path", "/stuff".as_bytes()),
                (":authority", "example.org".as_bytes()),
            ]
            .as_ref()
        )
        .as_slice(),
    );
    assert_eq!(pending.request.body.as_bytes(), b"example body");
    assert_eq!(
        pending.request.trailers.as_slice(),
        HeaderMap::from(
            [
                ("grpc-status", "0".as_bytes()),
                ("grpc-message", "OK".as_bytes()),
            ]
            .as_ref()
        )
        .as_slice(),
    );
    assert_eq!(pending.request.timeout, Duration::from_secs(3));

    Ok(())
}
