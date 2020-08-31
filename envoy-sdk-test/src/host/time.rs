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
//! #### Choosing initial time of the [`FakeClock`]:
//!
//! ```
//! # use envoy_sdk_test as envoy_test;
//! use std::time::{Duration, SystemTime};
//! use envoy::host::Clock;
//! use envoy_test::FakeClock;
//!
//! # fn main() -> envoy::host::Result<()> {
//! let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);
//!
//! let mut clock = FakeClock::new(t0);
//!
//! assert_eq!(clock.now()?, t0);
//! assert_eq!(clock.now()?, t0);
//! # Ok(())
//! # }
//! ```
//!
//! #### Advancing time of [`FakeClock`]:
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
//! assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
//! assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
//!
//! clock.advance(Duration::from_secs(3));
//!
//! let t1 = SystemTime::UNIX_EPOCH + Duration::from_secs(3);
//!
//! assert_eq!(clock.now()?, t1);
//! assert_eq!(clock.now()?, t1);
//! # Ok(())
//! # }
//! ```
//!
//! [`FakeClock`]: struct.FakeClock.html

use std::cell::RefCell;
use std::time::{Duration, SystemTime};

use envoy::host::time::Clock;
use envoy::host::Result;

/// Fake `System Clock`.
pub struct FakeClock(RefCell<SystemTime>);

impl Clock for FakeClock {
    /// Returns current system time.
    fn now(&self) -> Result<SystemTime> {
        Ok(*self.0.borrow())
    }
}

impl Default for FakeClock {
    /// Returns a clock freezed at [`UNIX_EPOCH`] time.
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
    /// let clock = FakeClock::default();
    ///
    /// assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
    /// assert_eq!(clock.now()?, SystemTime::UNIX_EPOCH);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`UNIX_EPOCH`]: https://doc.rust-lang.org/std/time/constant.UNIX_EPOCH.html
    fn default() -> Self {
        Self::new(SystemTime::UNIX_EPOCH)
    }
}

impl FakeClock {
    /// Returns a new `FakeClock` freezed at given time.
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
    /// let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);
    ///
    /// let clock = FakeClock::new(t0);
    ///
    /// assert_eq!(clock.now()?, t0);
    /// assert_eq!(clock.now()?, t0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(current_time: SystemTime) -> Self {
        FakeClock(RefCell::new(current_time))
    }

    /// Advances time forward by the specified `duration`.
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
    /// let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(5);
    ///
    /// let mut clock = FakeClock::new(t0);
    ///
    /// assert_eq!(clock.now()?, t0);
    /// assert_eq!(clock.now()?, t0);
    ///
    /// clock.advance(Duration::from_secs(3));
    ///
    /// let t1 = t0 + Duration::from_secs(3);
    ///
    /// assert_eq!(clock.now()?, t1);
    /// assert_eq!(clock.now()?, t1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn advance(&self, duration: Duration) -> &Self {
        let now = *self.0.borrow();
        self.0.replace(now + duration);
        self
    }
}
