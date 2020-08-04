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

//! Fake `Time API`.
//!
//! # Examples
//!
//! #### Basic usage of [`FakeClock`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use std::time::{Duration, SystemTime};
//! use envoy::host::Clock;
//! use envoy_test::FakeClock;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let clock = FakeClock::default();
//!
//! assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
//! assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
//! # Ok(())
//! # }
//! ```
//!
//! #### Setting a time of the [`FakeClock`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use std::time::{Duration, SystemTime};
//! use envoy::host::Clock;
//! use envoy_test::FakeClock;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let mut clock = FakeClock::default();
//!
//! let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);
//!
//! clock.freeze_at(t0);
//!
//! assert_eq!(clock.now()?, t0);
//! assert_eq!(clock.now()?, t0);
//! # Ok(())
//! # }
//! ```
//!
//! #### Setting a sequence of time values of the [`FakeClock`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use std::time::{Duration, SystemTime};
//! use envoy::host::Clock;
//! use envoy_test::FakeClock;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let mut clock = FakeClock::default();
//!
//! let t0 = SystemTime::UNIX_EPOCH;
//! let moments = (0..).map(move |i| t0 + Duration::from_secs(i));
//!
//! clock.tick_at(moments);
//!
//! assert_eq!(clock.now()?, t0);
//! assert_eq!(clock.now()?, t0 + Duration::from_secs(1));
//! assert_eq!(clock.now()?, t0 + Duration::from_secs(2));
//! # Ok(())
//! # }
//! ```
//!
//! [`FakeClock`]: struct.FakeClock.html

use std::cell::RefCell;
use std::iter;
use std::time::SystemTime;

use envoy::host::time::Clock;
use envoy::host::Result;

/// Fake `System Clock`.
pub struct FakeClock(RefCell<Box<dyn Iterator<Item = SystemTime>>>);

impl Clock for FakeClock {
    /// Returns current system time.
    fn now(&self) -> Result<SystemTime> {
        self.0
            .borrow_mut()
            .next()
            .ok_or_else(|| "no more ticks".into())
    }
}

impl Default for FakeClock {
    /// Returns a clock freezed at [`UNIX_EPOCH`] time.
    ///
    /// [`UNIX_EPOCH`]: https://doc.rust-lang.org/std/time/constant.UNIX_EPOCH.html
    fn default() -> Self {
        FakeClock::new(iter::repeat(SystemTime::UNIX_EPOCH))
    }
}

impl FakeClock {
    fn new<T>(ticker: T) -> Self
    where
        T: IntoIterator<Item = SystemTime>,
        T::IntoIter: 'static,
    {
        FakeClock(RefCell::new(Box::new(ticker.into_iter())))
    }

    /// Sets a time value to be returned by the subsequent calls to `now()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use envoy_sdk_test as envoy_test;
    /// use std::time::{Duration, SystemTime};
    /// use envoy::host::Clock;
    /// use envoy_test::FakeClock;
    ///
    /// # fn main() -> envoy::host::Result<()> {
    /// let mut clock = FakeClock::default();
    ///
    /// let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);
    ///
    /// clock.freeze_at(t0);
    ///
    /// assert_eq!(clock.now()?, t0);
    /// assert_eq!(clock.now()?, t0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn freeze_at(&self, time: SystemTime) -> &Self {
        drop(self.0.replace(Box::new(iter::repeat(time))));
        self
    }

    /// Sets a sequence of time values to be returned by the subsequent calls to `now()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use envoy_sdk_test as envoy_test;
    /// use std::time::{Duration, SystemTime};
    /// use envoy::host::Clock;
    /// use envoy_test::FakeClock;
    ///
    /// # fn main() -> envoy::host::Result<()> {
    /// let mut clock = FakeClock::default();
    ///
    /// let t0 = SystemTime::UNIX_EPOCH;
    /// let moments = (0..).map(move |i| t0 + Duration::from_secs(i));
    ///
    /// clock.tick_at(moments);
    ///
    /// assert_eq!(clock.now()?, t0);
    /// assert_eq!(clock.now()?, t0 + Duration::from_secs(1));
    /// assert_eq!(clock.now()?, t0 + Duration::from_secs(2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn tick_at<T>(&self, ticker: T) -> &Self
    where
        T: IntoIterator<Item = SystemTime>,
        T::IntoIter: 'static,
    {
        drop(self.0.replace(Box::new(ticker.into_iter())));
        self
    }
}
