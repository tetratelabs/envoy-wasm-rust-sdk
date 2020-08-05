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

//! `Envoy` `Stats API`.

use std::ops::Deref;
use std::rc::Rc;

use crate::host;

/// An interface of the `Envoy` `Stats API`.
///
/// # Examples
///
/// #### Basic usage of [`Stats`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::Stats;
///
/// let stats = Stats::default();
///
/// let requests_total = stats.counter("requests_total")?;
///
/// requests_total.inc();
/// # Ok(())
/// # }
/// ```
///
/// #### Injecting [`Stats`] into a HTTP Filter as a dependency:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::host::Stats;
///
/// struct MyHttpFilter<'a> {
///     stats: &'a dyn Stats,
/// }
///
/// impl<'a> MyHttpFilter<'a> {
///     /// Creates a new instance parameterized with a given [`Stats`] implementation.
///     pub fn new(stats: &'a dyn Stats) -> Self {
///         MyHttpFilter { stats }
///     }
///
///     /// Creates a new instance parameterized with the default [`Stats`] implementation.
///     pub fn default() -> Self {
///         Self::new(Stats::default())
///     }
/// }
/// ```
///
/// [`Stats`]: trait.Stats.html
pub trait Stats {
    /// Creates a [`Counter`] from the stat name.
    ///
    /// Tag extraction will be performed on the name.
    ///
    /// [`Counter`]: trait.Counter.html
    fn counter(&self, name: &str) -> host::Result<Box<dyn Counter>>;

    /// Creates a [`Gauge`] from the stat name.
    ///
    /// Tag extraction will be performed on the name.
    ///
    /// [`Gauge`]: trait.Gauge.html
    fn gauge(&self, name: &str) -> host::Result<Box<dyn Gauge>>;

    /// Creates a [`Histogram`] from the stat name.
    ///
    /// Tag extraction will be performed on the name.
    ///
    /// [`Histogram`]: trait.Histogram.html
    fn histogram(&self, name: &str) -> host::Result<Box<dyn Histogram>>;
}

/// An interface of the `Envoy` `Counter`.
///
/// A `Counter` can only be incremented.
///
/// # Examples
///
/// #### Basic usage of [`Counter`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::Stats;
///
/// let stats = Stats::default();
///
/// let requests_total = stats.counter("requests_total")?;
///
/// requests_total.inc()?;
/// # Ok(())
/// # }
/// ```
///
/// [`Counter`]: trait.Counter.html
pub trait Counter {
    /// Increments counter by `1`.
    fn inc(&self) -> host::Result<()> {
        self.add(1)
    }
    /// Increments counter by a given offset.
    fn add(&self, offset: u64) -> host::Result<()>;
    /// Returns current value of the counter.
    fn value(&self) -> host::Result<u64>;
}

/// An interface of the `Envoy` `Gauge`.
///
/// A `Gauge` can be both incremented and decremented.
///
/// # Examples
///
/// #### Basic usage of [`Gauge`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::Stats;
///
/// let stats = Stats::default();
///
/// let requests_active = stats.gauge("requests_active")?;
///
/// requests_active.inc()?;
///
/// # stringify! {
/// ... do some work ...
/// # };
///
/// requests_active.dec()?;
/// # Ok(())
/// # }
/// ```
///
/// [`Gauge`]: trait.Gauge.html
pub trait Gauge {
    /// Increments gauge by `1`.
    fn inc(&self) -> host::Result<()> {
        self.add(1)
    }
    /// Decrements gauge by `1`.
    fn dec(&self) -> host::Result<()> {
        self.sub(1)
    }
    /// Increments gauge by a given offset.
    fn add(&self, offset: u64) -> host::Result<()>;
    /// Decrements gauge by a given offset.
    fn sub(&self, offset: u64) -> host::Result<()>;
    /// Sets gauge to a given value.
    fn set(&self, value: u64) -> host::Result<()>;
    /// Returns current value of the gauge.
    fn value(&self) -> host::Result<u64>;
}

/// An interface of the `Envoy` `Histogram`.
///
/// A `Histogram` records values one at a time.
///
/// # Examples
///
/// #### Basic usage of [`Histogram`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::Stats;
///
/// let stats = Stats::default();
///
/// let response_times_millis = stats.histogram("response_times_millis")?;
///
/// response_times_millis.record(123)?;
/// # Ok(())
/// # }
/// ```
///
/// [`Histogram`]: trait.Histogram.html
pub trait Histogram {
    /// Records a given value.
    fn record(&self, value: u64) -> host::Result<()>;
}

impl dyn Stats {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn Stats {
        &impls::Host
    }
}

impl<T: Counter> Counter for Rc<T> {
    /// Increments counter by a given offset.
    fn add(&self, offset: u64) -> host::Result<()> {
        self.deref().add(offset)
    }
    /// Returns current value of the counter.
    fn value(&self) -> host::Result<u64> {
        self.deref().value()
    }
}

impl<T: Gauge> Gauge for Rc<T> {
    /// Increments gauge by a given offset.
    fn add(&self, offset: u64) -> host::Result<()> {
        self.deref().add(offset)
    }
    /// Decrements gauge by a given offset.
    fn sub(&self, offset: u64) -> host::Result<()> {
        self.deref().sub(offset)
    }
    /// Sets gauge to a given value.
    fn set(&self, value: u64) -> host::Result<()> {
        self.deref().set(value)
    }
    /// Returns current value of the gauge.
    fn value(&self) -> host::Result<u64> {
        self.deref().value()
    }
}

impl<T: Histogram> Histogram for Rc<T> {
    /// Records a given value.
    fn record(&self, value: u64) -> host::Result<()> {
        self.deref().record(value)
    }
}

mod impls {
    use std::cmp;

    use super::Stats;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::{MetricHandle, MetricType};
    use crate::host;

    pub(super) struct Host;

    impl Stats for Host {
        fn counter(&self, name: &str) -> host::Result<Box<dyn super::Counter>> {
            hostcalls::define_metric(MetricType::Counter, name)
                .map(|handle| Box::new(Counter(handle)) as Box<dyn super::Counter>)
        }

        fn gauge(&self, name: &str) -> host::Result<Box<dyn super::Gauge>> {
            hostcalls::define_metric(MetricType::Gauge, name)
                .map(|handle| Box::new(Gauge(handle)) as Box<dyn super::Gauge>)
        }

        fn histogram(&self, name: &str) -> host::Result<Box<dyn super::Histogram>> {
            hostcalls::define_metric(MetricType::Histogram, name)
                .map(|handle| Box::new(Histogram(handle)) as Box<dyn super::Histogram>)
        }
    }

    struct Counter(MetricHandle);

    impl super::Counter for Counter {
        fn add(&self, offset: u64) -> host::Result<()> {
            let mut offset = offset;
            while 0 < offset {
                let delta = cmp::min(offset, std::i64::MAX as u64) as i64;
                if let Err(err) = hostcalls::increment_metric(self.0, delta) {
                    return Err(err);
                }
                offset -= delta as u64;
            }
            Ok(())
        }

        fn value(&self) -> host::Result<u64> {
            hostcalls::get_metric(self.0)
        }
    }

    struct Gauge(MetricHandle);

    impl super::Gauge for Gauge {
        fn add(&self, offset: u64) -> host::Result<()> {
            let mut offset = offset;
            while 0 < offset {
                let delta = cmp::min(offset, std::i64::MAX as u64) as i64;
                if let Err(err) = hostcalls::increment_metric(self.0, delta) {
                    return Err(err);
                }
                offset -= delta as u64;
            }
            Ok(())
        }

        fn sub(&self, offset: u64) -> host::Result<()> {
            let mut offset = offset;
            while 0 < offset {
                let delta = cmp::min(offset, std::i64::MAX as u64) as i64;
                if let Err(err) = hostcalls::increment_metric(self.0, -delta) {
                    return Err(err);
                }
                offset -= delta as u64;
            }
            Ok(())
        }

        fn set(&self, value: u64) -> host::Result<()> {
            hostcalls::record_metric(self.0, value)
        }

        fn value(&self) -> host::Result<u64> {
            hostcalls::get_metric(self.0)
        }
    }

    struct Histogram(MetricHandle);

    impl super::Histogram for Histogram {
        fn record(&self, value: u64) -> host::Result<()> {
            hostcalls::record_metric(self.0, value)
        }
    }
}
