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

use crate::host;

pub trait Counter {
    fn inc(&self) -> host::Result<()> {
        self.add(1)
    }
    fn add(&self, offset: u64) -> host::Result<()>;
    fn value(&self) -> host::Result<u64>;
}

pub trait Gauge {
    fn inc(&self) -> host::Result<()> {
        self.add(1)
    }
    fn dec(&self) -> host::Result<()> {
        self.sub(1)
    }
    fn add(&self, offset: u64) -> host::Result<()>;
    fn sub(&self, offset: u64) -> host::Result<()>;
    fn set(&self, value: u64) -> host::Result<()>;
    fn value(&self) -> host::Result<u64>;
}

pub trait Histogram {
    fn record(&self, value: u64) -> host::Result<()>;
}

pub trait Service {
    fn counter(&self, name: &str) -> host::Result<Box<dyn Counter>>;
    fn gauge(&self, name: &str) -> host::Result<Box<dyn Gauge>>;
    fn histogram(&self, name: &str) -> host::Result<Box<dyn Histogram>>;
}

impl dyn Service {
    pub fn default() -> &'static dyn Service {
        &impls::Host
    }
}

mod impls {
    use std::cmp;

    use super::Service;
    use crate::abi::proxy_wasm::hostcalls;
    use crate::abi::proxy_wasm::types::{MetricHandle, MetricType};
    use crate::host;

    pub(super) struct Host;

    impl Service for Host {
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
