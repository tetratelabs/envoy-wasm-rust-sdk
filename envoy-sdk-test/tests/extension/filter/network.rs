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

use envoy::extension::filter::network::{self, BufferAction, FilterStatus};
use envoy::extension::{self, ExtensionFactory, InstanceId, NetworkFilter};
use envoy::host::{Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeEnvoy;

#[test]
fn test_stats() -> Result<()> {
    struct TestFilter<'a> {
        stats: &'a dyn Stats,
    }
    impl NetworkFilter for TestFilter<'_> {
        fn on_new_connection(&mut self) -> extension::Result<network::FilterStatus> {
            self.stats.counter("test_filter.cx_total")?.inc()?;
            Ok(network::FilterStatus::Continue)
        }

        fn on_downstream_data(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn network::DownstreamDataOps,
        ) -> extension::Result<network::FilterStatus> {
            if data_size > 0 {
                let mut data = ops.downstream_data(0, data_size)?.into_vec();
                if !data.is_empty() {
                    data.remove(0);
                }
                ops.mutate_downstream_data(BufferAction::replace_with(&data))?;
            }
            Ok(network::FilterStatus::Continue)
        }

        fn on_upstream_data(
            &mut self,
            data_size: usize,
            _end_of_stream: bool,
            ops: &dyn network::UpstreamDataOps,
        ) -> extension::Result<network::FilterStatus> {
            if data_size > 0 {
                let mut data = ops.upstream_data(0, data_size)?.into_vec();
                data.extend("!".bytes());
                ops.mutate_upstream_data(BufferAction::replace_with(&data))?;
            }
            Ok(network::FilterStatus::Continue)
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
        .tcp()
        .network_filter(TestFilterFactory::new(&fake.stats))?
        .configure("{}")?;

    let mut connection = fake_listener.new_connection()?;
    {
        let status = connection.receive_connect_from_downstream()?;

        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 1);
        assert_eq!(connection.upstream().received_connect(), true);

        let status = connection.receive_data_from_downstream(b"hello")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(*connection.downstream_read_buffer(), b"");
        assert_eq!(connection.upstream().drain_received_bytes(), b"ello");

        let status = connection.receive_data_from_downstream(b"world")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(*connection.downstream_read_buffer(), b"");
        assert_eq!(connection.upstream().drain_received_bytes(), b"orld");

        let status = connection.receive_close_from_downstream()?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.upstream().received_close(), true);

        let status = connection.receive_data_from_upstream(b"hi")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(*connection.upstream_read_buffer(), b"");
        assert_eq!(connection.downstream().drain_received_bytes(), b"hi!");

        let status = connection.receive_data_from_upstream(b"there")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(*connection.upstream_read_buffer(), b"");
        assert_eq!(connection.downstream().drain_received_bytes(), b"there!");

        let status = connection.receive_close_from_upstream()?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.downstream().received_close(), true);
    }

    let mut connection2 = fake_listener.new_connection()?;
    {
        let status = connection2.receive_connect_from_downstream()?;
        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 2);
    }

    Ok(())
}
