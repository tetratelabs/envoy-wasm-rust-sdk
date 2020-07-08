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

use envoy::host::stats::{Counter, Gauge, Histogram};

// Sample stats.
pub struct SampleNetworkFilterStats {
    requests_total: Box<dyn Counter>,
    requests_active: Box<dyn Gauge>,
    response_body_size_bytes: Box<dyn Histogram>,
}

impl SampleNetworkFilterStats {
    pub fn new(
        requests_total: Box<dyn Counter>,
        requests_active: Box<dyn Gauge>,
        response_body_size_bytes: Box<dyn Histogram>,
    ) -> Self {
        SampleNetworkFilterStats {
            requests_total,
            requests_active,
            response_body_size_bytes,
        }
    }

    pub fn requests_total(&self) -> &dyn Counter {
        &*self.requests_total
    }
    pub fn requests_active(&self) -> &dyn Gauge {
        &*self.requests_active
    }
    pub fn response_body_size_bytes(&self) -> &dyn Histogram {
        &*self.response_body_size_bytes
    }
}
