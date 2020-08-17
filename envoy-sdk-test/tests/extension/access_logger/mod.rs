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

use envoy::extension::{access_logger, AccessLogger};
use envoy::host::{Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::{FakeEnvoy, FakeStreamInfo};

#[test]
fn test_access_logger() -> Result<()> {
    struct TestAccessLogger<'a> {
        stats: &'a dyn Stats,
    }
    impl AccessLogger for TestAccessLogger<'_> {
        fn name() -> &'static str {
            "test_access_logger"
        }

        fn on_log(&mut self, _ops: &dyn access_logger::LogOps) -> Result<()> {
            self.stats
                .counter("test_access_logger.log_entries_total")?
                .inc()?;
            Ok(())
        }
    }
    impl<'a> TestAccessLogger<'a> {
        fn new(stats: &'a dyn Stats) -> Self {
            TestAccessLogger { stats }
        }
    }

    let fake = FakeEnvoy::default();
    let mut fake_access_log = fake
        .access_log()
        .logger(TestAccessLogger::new(&fake.stats))
        .configure("{}")?;

    fake_access_log.log_connection(&FakeStreamInfo::default())?;

    fake_access_log.log_http_request(&FakeStreamInfo::default())?;

    Ok(())
}
