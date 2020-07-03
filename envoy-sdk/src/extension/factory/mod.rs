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

extern crate std;
use std::prelude::v1::*;

pub mod context;
pub mod ops;
pub use context::FactoryContext;

use crate::extension::{InstanceId, Result};
use crate::host;

use proxy_wasm::types::Bytes;

pub trait Factory {
    type Extension;

    const NAME: &'static str;

    fn on_configure(
        &mut self,
        _configuration_size: usize,
        _ops: &dyn ConfigureOps,
    ) -> Result<bool> {
        Ok(true)
    }

    fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension>;

    fn on_drain(&mut self, _ops: &dyn DrainOps) -> Result<bool> {
        Ok(true)
    }
}

pub trait ConfigureOps {
    fn get_configuration(&self) -> host::Result<Option<Bytes>>;
}

pub trait DrainOps {
    fn done(&self) -> host::Result<()>;
}

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
