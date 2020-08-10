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

//! `Envoy` `Shared Data API`.

use crate::host::{self, Bytes};

pub use crate::abi::proxy_wasm::types::OptimisticLockVersion;

/// An interface of the `Envoy` `Shared Data API`.
///
/// Basic usage of [`SharedData`]:
///
/// ```
/// # use envoy_sdk as envoy;
/// # use envoy::host::Result;
/// # fn action() -> Result<()> {
/// use envoy::host::SharedData;
///
/// let shared_data = SharedData::default();
///
/// let value = shared_data.get("shared_key")?;
///
/// shared_data.set("shared_key", b"shared value", None)?;
/// # Ok(())
/// # }
/// ```
///
/// Injecting [`SharedData`] into a HTTP Filter as a dependency:
///
/// ```
/// # use envoy_sdk as envoy;
/// use envoy::host::SharedData;
///
/// struct MyHttpFilter<'a> {
///     shared_data: &'a dyn SharedData,
/// }
///
/// impl<'a> MyHttpFilter<'a> {
///     /// Creates a new instance parameterized with a given [`SharedData`] implementation.
///     pub fn new(shared_data: &'a dyn SharedData) -> Self {
///         MyHttpFilter { shared_data }
///     }
///
///     /// Creates a new instance parameterized with the default [`SharedData`] implementation.
///     pub fn default() -> Self {
///         Self::new(SharedData::default())
///     }
/// }
/// ```
///
/// [`SharedData`]: trait.SharedData.html
pub trait SharedData {
    /// Returns shared data by key.
    ///
    /// # Arguments
    ///
    /// * `key` - key.
    ///
    /// # Return value
    ///
    /// * `value`   - an opaque blob of bytes.
    /// * `version` - optimistic lock version.
    fn get(&self, key: &str) -> host::Result<(Option<Bytes>, Option<OptimisticLockVersion>)>;

    /// Shares data under a given key.
    ///
    /// # Arguments
    ///
    /// * `key`     - key.
    /// * `value`   - an opaque blob of bytes.
    /// * `version` - optimistic lock version.
    fn set(
        &self,
        key: &str,
        value: &[u8],
        version: Option<OptimisticLockVersion>,
    ) -> host::Result<()>;
}

impl dyn SharedData {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn SharedData {
        &impls::Host
    }
}

mod impls {
    use super::SharedData;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::OptimisticLockVersion;
    use crate::host::{self, Bytes};

    pub(super) struct Host;

    impl SharedData for Host {
        fn get(&self, key: &str) -> host::Result<(Option<Bytes>, Option<OptimisticLockVersion>)> {
            hostcalls::get_shared_data(key)
        }

        fn set(
            &self,
            key: &str,
            value: &[u8],
            version: Option<OptimisticLockVersion>,
        ) -> host::Result<()> {
            hostcalls::set_shared_data(key, value, version)
        }
    }
}
