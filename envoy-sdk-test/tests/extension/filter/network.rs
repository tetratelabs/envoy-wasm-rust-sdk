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

use envoy::extension::filter::network;
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

    let mut fake_connection = fake_listener.new_connection()?;
    {
        let status = fake_connection.open()?;
        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 1);

        fake_connection.send(b"test")?;
    }

    let mut fake_connection2 = fake_listener.new_connection()?;
    {
        let status = fake_connection2.open()?;
        assert_eq!(status, network::FilterStatus::Continue);
        assert_eq!(fake.stats.counter("test_filter.cx_total")?.value()?, 2);
    }

    Ok(())
}
