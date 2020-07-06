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

pub mod ops {
    use std::cmp;

    use super::Service;
    use crate::host;

    pub struct Host;

    impl Service for Host {
        fn counter(&self, name: &str) -> host::Result<Box<dyn super::Counter>> {
            abi::define_metric(abi::MetricType::Counter, name)
                .map(|handle| Box::new(Counter(handle)) as Box<dyn super::Counter>)
        }

        fn gauge(&self, name: &str) -> host::Result<Box<dyn super::Gauge>> {
            abi::define_metric(abi::MetricType::Gauge, name)
                .map(|handle| Box::new(Gauge(handle)) as Box<dyn super::Gauge>)
        }

        fn histogram(&self, name: &str) -> host::Result<Box<dyn super::Histogram>> {
            abi::define_metric(abi::MetricType::Histogram, name)
                .map(|handle| Box::new(Histogram(handle)) as Box<dyn super::Histogram>)
        }
    }

    struct Counter(abi::MetricHandle);

    impl super::Counter for Counter {
        fn add(&self, offset: u64) -> host::Result<()> {
            let mut offset = offset;
            while 0 < offset {
                let delta = cmp::min(offset, std::i64::MAX as u64) as i64;
                if let Err(err) = abi::increment_metric(self.0, delta) {
                    return Err(err);
                }
                offset -= delta as u64;
            }
            Ok(())
        }

        fn value(&self) -> host::Result<u64> {
            abi::get_metric(self.0)
        }
    }

    struct Gauge(abi::MetricHandle);

    impl super::Gauge for Gauge {
        fn add(&self, offset: u64) -> host::Result<()> {
            let mut offset = offset;
            while 0 < offset {
                let delta = cmp::min(offset, std::i64::MAX as u64) as i64;
                if let Err(err) = abi::increment_metric(self.0, delta) {
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
                if let Err(err) = abi::increment_metric(self.0, -delta) {
                    return Err(err);
                }
                offset -= delta as u64;
            }
            Ok(())
        }

        fn set(&self, value: u64) -> host::Result<()> {
            abi::record_metric(self.0, value)
        }

        fn value(&self) -> host::Result<u64> {
            abi::get_metric(self.0)
        }
    }

    struct Histogram(abi::MetricHandle);

    impl super::Histogram for Histogram {
        fn record(&self, value: u64) -> host::Result<()> {
            abi::record_metric(self.0, value)
        }
    }

    mod abi {
        use proxy_wasm::types::Status;

        use crate::host;

        #[repr(u32)]
        #[derive(Debug)]
        pub enum MetricType {
            Counter = 0,
            Gauge = 1,
            Histogram = 2,
        }

        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        pub struct MetricHandle(u32);

        impl From<u32> for MetricHandle {
            fn from(metric_id: u32) -> Self {
                MetricHandle(metric_id)
            }
        }

        extern "C" {
            fn proxy_define_metric(
                metric_type: MetricType,
                metric_name_data: *const u8,
                metric_name_size: usize,
                return_metric_id: *mut u32,
            ) -> Status;
        }

        pub fn define_metric(
            metric_type: MetricType,
            metric_name: &str,
        ) -> host::Result<MetricHandle> {
            unsafe {
                let mut return_metric_id: u32 = 0;
                match proxy_define_metric(
                    metric_type,
                    metric_name.as_ptr(),
                    metric_name.len(),
                    &mut return_metric_id,
                ) {
                    Status::Ok => Ok(MetricHandle::from(return_metric_id)),
                    status => Err(("proxy_define_metric", status)),
                }
            }
        }

        extern "C" {
            fn proxy_increment_metric(metric_id: u32, offset: i64) -> Status;
        }

        pub fn increment_metric(metric_handle: MetricHandle, offset: i64) -> host::Result<()> {
            unsafe {
                match proxy_increment_metric(metric_handle.0, offset) {
                    Status::Ok => Ok(()),
                    status => Err(("proxy_increment_metric", status)),
                }
            }
        }

        extern "C" {
            fn proxy_record_metric(metric_id: u32, value: u64) -> Status;
        }

        pub fn record_metric(metric_handle: MetricHandle, value: u64) -> host::Result<()> {
            unsafe {
                match proxy_record_metric(metric_handle.0, value) {
                    Status::Ok => Ok(()),
                    status => Err(("proxy_record_metric", status)),
                }
            }
        }

        extern "C" {
            fn proxy_get_metric(metric_id: u32, return_metric_value: *mut u64) -> Status;
        }

        pub fn get_metric(metric_handle: MetricHandle) -> host::Result<u64> {
            unsafe {
                let mut return_metric_value: u64 = 0;
                match proxy_get_metric(metric_handle.0, &mut return_metric_value) {
                    Status::Ok => Ok(return_metric_value),
                    status => Err(("proxy_increment_metric", status)),
                }
            }
        }
    }
}
