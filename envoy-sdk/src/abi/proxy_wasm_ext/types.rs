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

use std::fmt;

pub use proxy_wasm::types::*;

// HTTP Client API

/// Opaque identifier of a request made via `HTTP Client API`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct HttpRequestHandle(u32);

impl From<u32> for HttpRequestHandle {
    fn from(token_id: u32) -> Self {
        HttpRequestHandle(token_id)
    }
}

impl fmt::Display for HttpRequestHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Shared Queue API

/// Opaque identifier of a queue accessible via `Shared Queue API`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SharedQueueHandle(u32);

impl SharedQueueHandle {
    pub(crate) fn as_id(&self) -> u32 {
        self.0
    }
}

impl From<u32> for SharedQueueHandle {
    fn from(token_id: u32) -> Self {
        SharedQueueHandle(token_id)
    }
}

impl fmt::Display for SharedQueueHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Stats API

/// Metric type, i.e. `Counter`, `Gauge` or `Histogram`.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MetricType {
    Counter = 0,
    Gauge = 1,
    Histogram = 2,
}

/// Opaque identifier of a metric accessible via `Stats API`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MetricHandle(u32);

impl MetricHandle {
    pub(crate) fn as_id(&self) -> u32 {
        self.0
    }
}

impl From<u32> for MetricHandle {
    fn from(metric_id: u32) -> Self {
        MetricHandle(metric_id)
    }
}
