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

use envoy::host::{Result, Stats};

use envoy_sdk_test as envoy_test;
use envoy_test::host::FakeStats;

#[test]
fn test_fake_counter() -> Result<()> {
    let stats = FakeStats::default();

    let counter = stats.counter("my.counter")?;
    assert_eq!(counter.value()?, 0);

    counter.inc()?;
    assert_eq!(counter.value()?, 1);

    counter.inc()?;
    assert_eq!(counter.value()?, 2);

    counter.add(3)?;
    assert_eq!(counter.value()?, 5);

    assert_eq!(stats.counter("my.counter")?.value()?, 5);

    let other_counter = stats.counter("my.other_counter")?;
    assert_eq!(other_counter.value()?, 0);

    other_counter.add(3)?;
    assert_eq!(other_counter.value()?, 3);

    assert_eq!(stats.counter("my.counter")?.value()?, 5);

    Ok(())
}

#[test]
fn test_fake_gauge() -> Result<()> {
    let stats = FakeStats::default();

    let gauge = stats.gauge("my.gauge")?;
    assert_eq!(gauge.value()?, 0);

    gauge.inc()?;
    assert_eq!(gauge.value()?, 1);

    gauge.inc()?;
    assert_eq!(gauge.value()?, 2);

    gauge.add(3)?;
    assert_eq!(gauge.value()?, 5);

    gauge.dec()?;
    assert_eq!(gauge.value()?, 4);

    gauge.sub(3)?;
    assert_eq!(gauge.value()?, 1);

    assert_eq!(stats.gauge("my.gauge")?.value()?, 1);

    let other_gauge = stats.gauge("my.other_gauge")?;
    assert_eq!(other_gauge.value()?, 0);

    other_gauge.add(3)?;
    assert_eq!(other_gauge.value()?, 3);

    assert_eq!(stats.gauge("my.gauge")?.value()?, 1);

    Ok(())
}

#[test]
fn test_fake_histogram() -> Result<()> {
    let stats = FakeStats::default();

    let histogram = stats.histogram("my.histogram")?;
    histogram.record(10)?;

    let other_histogram = stats.histogram("my.other_histogram")?;
    other_histogram.record(20)?;

    Ok(())
}
