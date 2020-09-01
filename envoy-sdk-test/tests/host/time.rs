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

use std::time::{Duration, SystemTime};

use envoy::host::{Clock, Result};

use envoy_sdk_test as envoy_test;
use envoy_test::FakeClock;

#[test]
fn test_default_fake_clock() -> Result<()> {
    let clock = FakeClock::default();

    assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
    assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);

    Ok(())
}

#[test]
fn test_fake_clock_at_given_time() -> Result<()> {
    let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);

    let clock = FakeClock::new(t0);

    assert_eq!(clock.now()?, t0);
    assert_eq!(clock.now()?, t0);

    Ok(())
}

#[test]
fn test_advance_fake_clock() -> Result<()> {
    let clock = FakeClock::default();

    assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
    assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);

    clock.advance(Duration::from_secs(3));

    let t1 = SystemTime::UNIX_EPOCH + Duration::from_secs(3);

    assert_eq!(clock.now()?, t1);
    assert_eq!(clock.now()?, t1);

    Ok(())
}
