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

//! `Envoy` `Access Logger API`.

use crate::abi::proxy_wasm_ext::types::Bytes;

use crate::extension::Result;
use crate::host;
use crate::host::http::client as http_client;

mod ops;

pub mod context;
pub use context::LoggerContext;

pub trait Logger {
    fn on_configure(
        &mut self,
        _configuration_size: usize,
        _logger_ops: &dyn ConfigureOps,
    ) -> Result<bool> {
        Ok(true)
    }

    fn on_log(&mut self, _logger_ops: &dyn LogOps) -> Result<()> {
        Ok(())
    }

    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        _request: http_client::RequestHandle,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
        _http_client_ops: &dyn http_client::ResponseOps,
    ) -> Result<()> {
        Ok(())
    }
}

pub trait ConfigureOps {
    fn get_configuration(&self) -> host::Result<Option<Bytes>>;
}

pub trait LogOps {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>>;

    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>>;

    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn get_property(&self, path: Vec<&str>) -> host::Result<Option<Bytes>>;
}

pub trait Ops: ConfigureOps + LogOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps;

    fn as_log_ops(&self) -> &dyn LogOps;
}

impl<T> Ops for T
where
    T: ConfigureOps + LogOps,
{
    fn as_configure_ops(&self) -> &dyn ConfigureOps {
        self
    }

    fn as_log_ops(&self) -> &dyn LogOps {
        self
    }
}

impl dyn Ops {
    pub fn default() -> &'static dyn Ops {
        &ops::Host
    }
}
