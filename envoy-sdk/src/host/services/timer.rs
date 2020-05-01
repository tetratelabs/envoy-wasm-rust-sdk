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
