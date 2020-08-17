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

    fake_access_log.log_http_request(&FakeStreamInfo::default())?;

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
