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

use envoy::extension::filter::network::FilterStatus;
use envoy::host::{Result, Stats};

use envoy_test::{FakeEnvoy, FakeHttpClientRequest, FakeHttpClientResponse};

use network_filter::SampleNetworkFilterFactory;

#[test]
fn test_network_filter() -> Result<()> {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(SampleNetworkFilterFactory::new(
            &fake.clock,
            &fake.http_client,
            &fake.stats,
        )?)
        .configure("{}")?;

    let mut connection = fake_listener.new_connection()?;
    {
        let status = connection.simulate_connect_from_downstream()?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(
            fake.stats
                .gauge("examples.network_filter.requests_active")?
                .value()?,
            1
        );
        assert_eq!(connection.upstream().received_connect(), false); // due to `FilterStatus::StopIteration`

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

        connection.simulate_http_client_response(
            pending.handle,
            FakeHttpClientResponse::builder()
                .header(":status", "200")
                .build(),
        )?;
        assert_eq!(connection.upstream().received_connect(), false); // due to absense of `resume` API
    }

    Ok(())
}
