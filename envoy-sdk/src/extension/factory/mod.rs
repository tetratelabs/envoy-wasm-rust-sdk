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

//! Extension Factory API.

use crate::abi::proxy_wasm::types::Bytes;

use crate::extension::{factory, InstanceId, Result};
use crate::host;

pub(crate) use self::context::ExtensionFactoryContext;

mod context;
mod ops;

/// Possible responses to the request to (re-)configure the extension.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConfigStatus {
    /// Extension has accepted the new configuration.
    Accepted,
    /// Extension has rejected the new configuration.
    Rejected,
}

impl ConfigStatus {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            ConfigStatus::Accepted => true,
            ConfigStatus::Rejected => false,
        }
    }
}

/// Possible responses to the request to drain the extension.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DrainStatus {
    /// Extension is being drained and cannot be removed just yet.
    Ongoing,
    /// Extension has been drained and can be removed now.
    Complete,
}

impl DrainStatus {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            DrainStatus::Ongoing => false,
            DrainStatus::Complete => true,
        }
    }
}

/// An interface of the `Envoy` extension `Factory`.
pub trait ExtensionFactory {
    type Extension;

    /// Name the extension should be referred to in `Envoy` configuration.
    const NAME: &'static str;

    /// Called when extension is being (re-)configured.
    ///
    /// # Arguments
    ///
    /// * `_configuration_size` - size of configuration data.
    /// * `_ops`                - a [`trait object`][`ConfigureOps`] through which extension `Factory` can access
    ///                           configuration.
    ///
    /// # Return value
    ///
    /// [`ConfigStatus`] telling `Envoy` whether configuration has been successfully applied.
    ///
    /// [`ConfigStatus`]: enum.ConfigStatus.html
    /// [`ConfigureOps`]: trait.ConfigureOps.html
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        _ops: &dyn factory::ConfigureOps,
    ) -> Result<ConfigStatus> {
        Ok(ConfigStatus::Accepted)
    }

    /// Called to create a new instance of the extension, e.g. [`HttpFilter`] or [`NetworkFilter`].
    ///
    /// # Arguments
    ///
    /// * `instance_id` - opaque identifier of the extension instance.
    ///
    /// # Return value
    ///
    /// a new instance of the extension.
    ///
    /// [`HttpFilter`]: ../filter/http/trait.HttpFilter.html
    /// [`NetworkFilter`]: ../filter/network/trait.NetworkFilter.html
    fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension>;

    /// Called when extension `Factory` is about to be destroyed.
    ///
    /// # Return value
    ///
    /// [`DrainStatus`] telling `Envoy` whether extension `Factory` has already been drained
    /// and can be now removed safely.
    ///
    /// [`DrainStatus`]: enum.DrainStatus.html
    fn on_drain(&mut self) -> Result<DrainStatus> {
        Ok(DrainStatus::Complete)
    }
}

/// An interface for accessing extension config.
pub trait ConfigureOps {
    fn configuration(&self) -> host::Result<Option<Bytes>>;
}

/// An interface for acknowledging `Envoy` that extension `Factory` has been drained.
pub trait DrainOps {
    fn done(&self) -> host::Result<()>;
}

#[doc(hidden)]
pub trait Ops: ConfigureOps + DrainOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps;

    fn as_done_ops(&self) -> &dyn DrainOps;
}

impl<T> Ops for T
where
    T: ConfigureOps + DrainOps,
{
    fn as_configure_ops(&self) -> &dyn ConfigureOps {
        self
    }

    fn as_done_ops(&self) -> &dyn DrainOps {
        self
    }
}

impl dyn Ops {
    /// Returns the default implementation that interacts with `Envoy`
    /// through its [`ABI`].
    ///
    /// [`ABI`]: https://github.com/proxy-wasm/spec
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
