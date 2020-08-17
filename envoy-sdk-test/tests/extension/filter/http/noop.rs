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

//! No-op `HTTP Filter` for use in unit tests.

use envoy::extension::{ExtensionFactory, HttpFilter, InstanceId, Result};

#[derive(Debug, Default)]
pub struct NoOpHttpFilter;

impl HttpFilter for NoOpHttpFilter {}

#[derive(Debug, Default)]
pub struct NoOpHttpFilterFactory;

impl ExtensionFactory for NoOpHttpFilterFactory {
    type Extension = NoOpHttpFilter;

    fn name() -> &'static str {
        "noop"
    }

    fn new_extension(&mut self, _instance_id: InstanceId) -> Result<Self::Extension> {
        Ok(NoOpHttpFilter)
    }
}
