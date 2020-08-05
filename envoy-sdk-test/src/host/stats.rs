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

//! Fake `Stats API`.
//!
//! # Examples
//!
//! #### Basic usage of [`FakeStats`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use envoy::host::Stats;
//! use envoy_test::FakeStats;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let stats = FakeStats::default();
//!
//! stats.counter("my.counter")?.inc()?;
//! stats.gauge("my.gauge")?.set(10)?;
//! stats.histogram("my.histogram")?.record(123)?;
//!
//! assert_eq!(stats.counter("my.counter")?.value()?, 1);
//! assert_eq!(stats.gauge("my.gauge")?.value()?, 10);
//! # Ok(())
//! # }
//! ```
//!
//! [`FakeStats`]: struct.FakeStats.html

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use envoy::host::stats::{Counter, Gauge, Histogram, Stats};
use envoy::host::Result;

/// Fake `Stats API`.
#[derive(Debug, Default)]
pub struct FakeStats {
    counters: RefCell<HashMap<String, Rc<FakeCounter>>>,
    gauges: RefCell<HashMap<String, Rc<FakeGauge>>>,
    histograms: RefCell<HashMap<String, Rc<FakeHistogram>>>,
}

/// Fake `Counter`.
#[derive(Debug, Default)]
struct FakeCounter(RefCell<u64>);

/// Fake `Gauge`.
#[derive(Debug, Default)]
struct FakeGauge(RefCell<u64>);

/// Fake `Histogram`.
#[derive(Debug, Default)]
struct FakeHistogram(RefCell<Vec<u64>>);

impl Stats for FakeStats {
    /// Creates a [`Counter`] from the stat name.
    fn counter(&self, name: &str) -> Result<Box<dyn Counter>> {
        let mut counters = self.counters.borrow_mut();
        let counter = counters
            .entry(name.to_string())
            .or_insert_with(|| Rc::new(FakeCounter::default()));
        Ok(Box::new(Rc::clone(counter)))
    }

    /// Creates a [`Gauge`] from the stat name.
    fn gauge(&self, name: &str) -> Result<Box<dyn Gauge>> {
        let mut gauges = self.gauges.borrow_mut();
        let gauge = gauges
            .entry(name.to_string())
            .or_insert_with(|| Rc::new(FakeGauge::default()));
        Ok(Box::new(Rc::clone(gauge)))
    }

    /// Creates a [`Histogram`] from the stat name.
    fn histogram(&self, name: &str) -> Result<Box<dyn Histogram>> {
        let mut histograms = self.histograms.borrow_mut();
        let histogram = histograms
            .entry(name.to_string())
            .or_insert_with(|| Rc::new(FakeHistogram::default()));
        Ok(Box::new(Rc::clone(histogram)))
    }
}

impl Counter for FakeCounter {
    /// Increments counter by a given offset.
    fn add(&self, offset: u64) -> Result<()> {
        *self.0.borrow_mut() += offset;
        Ok(())
    }

    /// Returns current value of the counter.
    fn value(&self) -> Result<u64> {
        Ok(*self.0.borrow())
    }
}

impl Gauge for FakeGauge {
    /// Increments gauge by a given offset.
    fn add(&self, offset: u64) -> Result<()> {
        *self.0.borrow_mut() += offset;
        Ok(())
    }

    /// Decrements gauge by a given offset.
    fn sub(&self, offset: u64) -> Result<()> {
        *self.0.borrow_mut() -= offset;
        Ok(())
    }

    /// Sets gauge to a given value.
    fn set(&self, value: u64) -> Result<()> {
        *self.0.borrow_mut() = value;
        Ok(())
    }

    /// Returns current value of the gauge.
    fn value(&self) -> Result<u64> {
        Ok(*self.0.borrow())
    }
}

impl Histogram for FakeHistogram {
    /// Records a given value.
    fn record(&self, value: u64) -> Result<()> {
        self.0.borrow_mut().push(value);
        Ok(())
    }
}

impl FakeStats {
    /// Resets all stats.
    pub fn reset(&self) {
        for counter in self.counters.borrow_mut().values() {
            counter.reset();
        }
        for gauge in self.gauges.borrow_mut().values() {
            gauge.reset();
        }
        for histogram in self.histograms.borrow_mut().values() {
            histogram.reset();
        }
    }
}

impl FakeCounter {
    /// Resets the counter to `0`.
    pub fn reset(&self) {
        *self.0.borrow_mut() = 0;
    }
}

impl FakeGauge {
    /// Resets the gauge to `0`.
    pub fn reset(&self) {
        *self.0.borrow_mut() = 0;
    }
}

impl FakeHistogram {
    /// Resets the histogram to empty.
    pub fn reset(&self) {
        self.0.replace(Vec::new());
    }
}
