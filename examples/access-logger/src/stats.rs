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

use envoy_sdk::host::services::metrics::{Counter, Gauge};

// Sample stats.
pub struct SampleAccessLoggerStats {
    requests_total: Box<dyn Counter>,
    reports_active: Box<dyn Gauge>,
    reports_total: Box<dyn Counter>,
}

impl SampleAccessLoggerStats {
    pub fn new(
        requests_total: Box<dyn Counter>,
        reports_active: Box<dyn Gauge>,
        reports_total: Box<dyn Counter>,
    ) -> SampleAccessLoggerStats {
        SampleAccessLoggerStats {
            requests_total,
            reports_active,
            reports_total,
        }
    }

    pub fn requests_total(&self) -> &dyn Counter {
        &*self.requests_total
    }
    pub fn reports_active(&self) -> &dyn Gauge {
        &*self.reports_active
    }
    pub fn reports_total(&self) -> &dyn Counter {
        &*self.reports_total
    }
}
