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
use std::time::Duration;

use crate::extension::Result;
use crate::host;

pub trait Service {
    fn set_tick_period(&self, period: Duration, ops: &dyn TimerOps) -> host::Result<()>;
}

pub trait TimerOps {
    fn on_tick(&mut self) -> Result<()>;
}
