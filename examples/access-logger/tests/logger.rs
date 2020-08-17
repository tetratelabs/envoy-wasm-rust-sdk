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

use envoy::host::stream_info::{ResponseFlags, TrafficDirection};
use envoy::host::{Result, Stats};

use envoy_test::{FakeEnvoy, FakeHttpClientRequest, FakeHttpClientResponse, FakeStreamInfo};

use access_logger::SampleAccessLogger;

#[test]
fn test_access_logger() -> Result<()> {
    let fake = FakeEnvoy::default();
    let mut fake_access_log = fake
        .access_log()
        .logger(SampleAccessLogger::new(
            &fake.clock,
            &fake.http_client,
            &fake.stats,
        )?)
        .configure("{}")?;

    let fake_http_request = FakeStreamInfo::new().with(|info| {
        info.connection()
            .id(123)
            .mtls(true)
            .requested_server_name("example.org");
        info.request()
            .id("a-b-c-d")
            .size(1024)
            .total_size(2048)
            .method("GET")
            .scheme("https")
            .host("www.example.com")
            .path("/search?q=example")
            .protocol("HTTP/1.1")
            .header("content-type", "application/json")
            .header("content-length", "1001")
            .header("user-agent", "curl")
            .header("referer", "https://www.example.com");
        info.response()
            .status_code(200)
            .header("content-type", "application/json")
            .header("content-length", "1001")
            .trailer("grpc-message", "UNKNOWN")
            .size(1024)
            .total_size(2048)
            .grpc_status(1)
            .response_flags(
                ResponseFlags::FAILED_LOCAL_HEALTH_CHECK | ResponseFlags::DELAY_INJECTED,
            );
        info.listener()
            .traffic_direction(TrafficDirection::OUTBOUND);
        info.route().name("my_route");
        info.cluster().name("my_cluster");
        info.upstream().address("192.168.0.1").port(5432);
    });

    fake_access_log.log(&fake_http_request)?;

    assert_eq!(
        fake.stats
            .counter("examples.access_logger.requests_total")?
            .value()?,
        1
    );
    assert_eq!(
        fake.stats
            .gauge("examples.access_logger.reports_active")?
            .value()?,
        1
    );
    assert_eq!(
        fake.stats
            .counter("examples.access_logger.reports_total")?
            .value()?,
        0
    );

    let pending_requests = fake.http_client.drain_pending_requests();
    assert_eq!(pending_requests.len(), 1);
    let pending = &pending_requests[0];

    assert_eq!(
        pending.request,
        FakeHttpClientRequest::builder()
            .upstream("mock_service")
            .header(":method", "GET")
            .header(":path", "/mock")
            .header(":authority", "mock.local")
            .timeout(Duration::from_secs(3))
            .build()
    );

    fake_access_log.simulate_http_client_response(
        pending.handle,
        FakeHttpClientResponse::builder()
            .header(":status", "200")
            .build(),
    )?;

    assert_eq!(
        fake.stats
            .gauge("examples.access_logger.reports_active")?
            .value()?,
        0
    );
    assert_eq!(
        fake.stats
            .counter("examples.access_logger.reports_total")?
            .value()?,
        1
    );

    Ok(())
}
