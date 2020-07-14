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

/// Possible outcomes of the extension (re-)configuration.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConfigStatus {
    /// Extension accepted the new configuration.
    Accepted,
    /// Extension rejected the new configuration.
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
