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

use envoy::extension::filter::http::{self, FilterDataStatus, FilterHeadersStatus};
use envoy::extension::{self, ExtensionFactory, HttpFilter, InstanceId};
use envoy::host::{BufferAction, Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeEnvoy;

#[test]
fn test_http_filter() -> Result<()> {
    struct TestFilter<'a> {
        stats: &'a dyn Stats,
    }
    impl HttpFilter for TestFilter<'_> {
        fn on_request_headers(
            &mut self,
            _num_headers: usize,
            _ops: &dyn http::RequestHeadersOps,
        ) -> Result<FilterHeadersStatus> {
            self.stats.counter("test_filter.requests_total")?.inc()?;
            Ok(FilterHeadersStatus::Continue)
        }

        fn on_request_body(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn http::RequestBodyOps,
        ) -> Result<FilterDataStatus> {
            if data_size > 0 {
                let mut data = ops.request_body(0, data_size)?.into_vec();
                if !data.is_empty() {
                    data.remove(0);
                }
                ops.mutate_request_body(BufferAction::replace_with(&data))?;
            }
            Ok(FilterDataStatus::Continue)
        }

        fn on_response_body(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn http::ResponseBodyOps,
        ) -> extension::Result<FilterDataStatus> {
            if data_size > 0 {
                let mut data = ops.response_body(0, data_size)?.into_vec();
                data.extend("!".bytes());
                ops.mutate_response_body(BufferAction::replace_with(&data))?;
            }
            Ok(FilterDataStatus::Continue)
        }
    }

    struct TestFilterFactory<'a> {
        stats: &'a dyn Stats,
    }
    impl<'a> ExtensionFactory for TestFilterFactory<'a> {
        type Extension = TestFilter<'a>;

        fn name() -> &'static str {
            "test"
        }

        fn new_extension(
            &mut self,
            _instance_id: InstanceId,
        ) -> extension::Result<Self::Extension> {
            Ok(TestFilter { stats: self.stats })
        }
    }
    impl<'a> TestFilterFactory<'a> {
        fn new(stats: &'a dyn Stats) -> Self {
            TestFilterFactory { stats }
        }
    }

    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .http()
        .http_filter(TestFilterFactory::new(&fake.stats))
        .configure("{}")?;

    let mut _stream = fake_listener.new_http_stream()?;
    {
        // let status = connection.simulate_connect_from_downstream()?;

        // assert_eq!(status, network::FilterStatus::Continue);
        // assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 1);
        // assert_eq!(connection.upstream().received_connect(), true);

        // let status = connection.simulate_data_from_downstream(b"hello")?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.peek_downstream_read_buffer(), b"");
        // assert_eq!(connection.upstream().drain_received_bytes(), b"ello");

        // let status = connection.simulate_data_from_downstream(b"world")?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.peek_downstream_read_buffer(), b"");
        // assert_eq!(connection.upstream().drain_received_bytes(), b"orld");

        // let status = connection.simulate_close_from_downstream()?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.upstream().received_close(), true);

        // let status = connection.simulate_data_from_upstream(b"hi")?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.downstream().drain_received_bytes(), b"hi!");

        // let status = connection.simulate_data_from_upstream(b"there")?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.downstream().drain_received_bytes(), b"there!");

        // let status = connection.simulate_close_from_upstream()?;

        // assert_eq!(status, FilterStatus::Continue);
        // assert_eq!(connection.downstream().received_close(), true);
    }

    let mut _stream2 = fake_listener.new_http_stream()?;
    {
        // let status = http_stream2.simulate_connect_from_downstream()?;
        // assert_eq!(status, network::FilterStatus::Continue);
        // assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 2);
    }

    Ok(())
}
