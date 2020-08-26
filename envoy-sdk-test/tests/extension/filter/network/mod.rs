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

use envoy::extension::filter::network::{self, FilterStatus};
use envoy::extension::{self, ExtensionFactory, InstanceId, NetworkFilter};
use envoy::host::buffer::Transform;
use envoy::host::{Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeEnvoy;

use self::noop::NoOpNetworkFilterFactory;

mod noop;

#[test]
fn test_network_filter() -> Result<()> {
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
                let mut data = ops.downstream_data(0, data_size)?.into_bytes();
                if !data.is_empty() {
                    data.remove(0);
                }
                ops.mutate_downstream_data(Transform::replace_with(&data))?;
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
                let mut data = ops.upstream_data(0, data_size)?.into_bytes();
                data.extend("!".bytes());
                ops.mutate_upstream_data(Transform::replace_with(&data))?;
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
        .network_filter(TestFilterFactory::new(&fake.stats))
        .configure("{}")?;

    let mut connection = fake_listener.new_connection()?;
    {
        let status = connection.simulate_connect_from_downstream()?;

        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 1);
        assert_eq!(connection.upstream().received_connect(), true);

        let status = connection.simulate_data_from_downstream(b"hello")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.peek_downstream_read_buffer(), b"");
        assert_eq!(connection.upstream().drain_received_bytes(), b"ello");

        let status = connection.simulate_data_from_downstream(b"world")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.peek_downstream_read_buffer(), b"");
        assert_eq!(connection.upstream().drain_received_bytes(), b"orld");

        let status = connection.simulate_close_from_downstream()?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.upstream().received_close(), true);

        let status = connection.simulate_data_from_upstream(b"hi")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.downstream().drain_received_bytes(), b"hi!");

        let status = connection.simulate_data_from_upstream(b"there")?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.downstream().drain_received_bytes(), b"there!");

        let status = connection.simulate_close_from_upstream()?;

        assert_eq!(status, FilterStatus::Continue);
        assert_eq!(connection.downstream().received_close(), true);
    }

    let mut connection2 = fake_listener.new_connection()?;
    {
        let status = connection2.simulate_connect_from_downstream()?;
        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 2);
    }

    Ok(())
}

#[test]
fn test_network_filter_downstream_stop_iteration() -> Result<()> {
    struct TestFilter<'a> {
        stats: &'a dyn Stats,
    }
    impl NetworkFilter for TestFilter<'_> {
        fn on_new_connection(&mut self) -> extension::Result<network::FilterStatus> {
            self.stats.counter("test_filter.on_new_connection")?.inc()?;
            Ok(network::FilterStatus::StopIteration)
        }

        fn on_downstream_data(
            &mut self,
            _data_size: usize,
            _end_of_stream: bool,
            _ops: &dyn network::DownstreamDataOps,
        ) -> extension::Result<network::FilterStatus> {
            self.stats
                .counter("test_filter.on_downstream_data")?
                .inc()?;
            Ok(network::FilterStatus::StopIteration)
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
        .network_filter(TestFilterFactory::new(&fake.stats))
        .configure("{}")?;

    {
        let mut connection = fake_listener.new_connection()?;

        let status = connection.simulate_connect_from_downstream()?;

        assert_eq!(status, network::FilterStatus::StopIteration);
        assert_eq!(
            fake.stats
                .counter("test_filter.on_new_connection")?
                .value()?,
            1
        );
        assert_eq!(connection.upstream().received_connect(), false); // because of StopIteration in `on_downstream_data`

        let status = connection.simulate_data_from_downstream(b"hello")?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(
            fake.stats
                .counter("test_filter.on_downstream_data")?
                .value()?,
            1
        );
        assert_eq!(connection.peek_downstream_read_buffer(), b"hello");
        assert_eq!(connection.upstream().received_connect(), false);
        assert_eq!(connection.upstream().drain_received_bytes(), b"");

        let status = connection.simulate_data_from_downstream(b"world")?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(
            fake.stats
                .counter("test_filter.on_downstream_data")?
                .value()?,
            2
        );
        assert_eq!(connection.peek_downstream_read_buffer(), b"helloworld");
        assert_eq!(connection.upstream().received_connect(), false);
        assert_eq!(connection.upstream().drain_received_bytes(), b"");
    }

    Ok(())
}

#[test]
fn test_network_filter_upstream_stop_iteration() -> Result<()> {
    struct TestFilter<'a> {
        stats: &'a dyn Stats,
    }
    impl NetworkFilter for TestFilter<'_> {
        fn on_new_connection(&mut self) -> extension::Result<network::FilterStatus> {
            self.stats.counter("test_filter.on_new_connection")?.inc()?;
            Ok(network::FilterStatus::Continue)
        }

        fn on_upstream_data(
            &mut self,
            _data_size: usize,
            _end_of_stream: bool,
            _ops: &dyn network::UpstreamDataOps,
        ) -> Result<FilterStatus> {
            self.stats.counter("test_filter.on_upstream_data")?.inc()?;
            Ok(network::FilterStatus::StopIteration)
        }

        fn on_upstream_close(
            &mut self,
            _peer_type: network::PeerType,
            _ops: &dyn network::UpstreamCloseOps,
        ) -> Result<()> {
            self.stats.counter("test_filter.on_upstream_close")?.inc()?;
            Ok(())
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
        .network_filter(TestFilterFactory::new(&fake.stats))
        .configure("{}")?;

    {
        let mut connection = fake_listener.new_connection()?;

        let status = connection.simulate_connect_from_downstream()?;

        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(
            fake.stats
                .counter("test_filter.on_new_connection")?
                .value()?,
            1
        );
        assert_eq!(connection.upstream().received_connect(), true);

        let status = connection.simulate_data_from_upstream(b"hi")?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(connection.downstream().drain_received_bytes(), b"");

        let status = connection.simulate_data_from_upstream(b"there")?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(connection.downstream().drain_received_bytes(), b"");

        let status = connection.simulate_close_from_upstream()?;

        assert_eq!(status, FilterStatus::StopIteration);
        assert_eq!(connection.downstream().received_close(), false); // because of StopIteration in `on_upstream_data`
    }

    Ok(())
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream has already connected"
)]
fn test_downstream_connects_second_time() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    let status = connection.simulate_connect_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    connection.simulate_connect_from_downstream().unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot close connection for the second time"
)]
fn test_downstream_closes_second_time() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    let status = connection.simulate_connect_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_close_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    connection.simulate_close_from_downstream().unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: downstream cannot keep sending data after closing the connection"
)]
fn test_downstream_sends_data_after_closing_connection() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    let status = connection.simulate_connect_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_data_from_downstream(b"hello").unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_close_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    connection
        .simulate_data_from_downstream(b" world!")
        .unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot start sending data prior to receiving a connect"
)]
fn test_upstream_sends_data_prior_to_receiving_connect() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    connection.simulate_data_from_upstream(b"hello").unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot close connection prior to receiving a connect"
)]
fn test_upstream_closes_prior_to_receiving_connect() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    connection.simulate_close_from_upstream().unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot close connection for the second time"
)]
fn test_upstream_closes_second_time() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    let status = connection.simulate_connect_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_close_from_upstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    connection.simulate_close_from_upstream().unwrap();
}

#[test]
#[should_panic(
    expected = "unit test is trying to do something that actual Envoy would never do: upstream cannot keep sending data after closing the connection"
)]
fn test_upstream_sends_data_after_closing_connection() {
    let fake = FakeEnvoy::default();
    let mut fake_listener = fake
        .listener()
        .tcp()
        .network_filter(NoOpNetworkFilterFactory::default())
        .configure("{}")
        .unwrap();

    let mut connection = fake_listener.new_connection().unwrap();

    let status = connection.simulate_connect_from_downstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_data_from_upstream(b"hi").unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    let status = connection.simulate_close_from_upstream().unwrap();
    assert_eq!(status, network::FilterStatus::Continue);

    connection.simulate_data_from_upstream(b" there!").unwrap();
}
