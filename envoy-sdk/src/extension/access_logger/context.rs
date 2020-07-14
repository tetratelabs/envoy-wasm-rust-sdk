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

use super::{Logger, Ops};
use crate::abi::proxy_wasm_ext::traits::{Context, RootContext};
use crate::host::http::client as http_client;
use crate::host::log;

pub struct LoggerContext<'a, L>
where
    L: Logger,
{
    logger: L,
    logger_ops: &'a dyn Ops,
    http_client_ops: &'a dyn http_client::ResponseOps,
}

impl<'a, L> RootContext for LoggerContext<'a, L>
where
    L: Logger,
{
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        match self.logger.on_configure(
            plugin_configuration_size,
            self.logger_ops.as_configure_ops(),
        ) {
            Ok(success) => success,
            Err(err) => {
                log::error!("failed to configure extension \"{}\": {}", L::NAME, err);
                false
            }
        }
    }

    fn on_log(&mut self) {
        if let Err(err) = self.logger.on_log(self.logger_ops.as_log_ops()) {
            log::error!("failed to log a request: {}", err);
        }
    }
}

impl<'a, L> Context for LoggerContext<'a, L>
where
    L: Logger,
{
    // Http Client callbacks

    fn on_http_call_response(
        &mut self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        num_trailers: usize,
    ) {
        self.logger
            .on_http_call_response(
                http_client::RequestHandle::from(token_id),
                num_headers,
                body_size,
                num_trailers,
                self.http_client_ops,
            )
            .unwrap()
    }
}

impl<'a, L> LoggerContext<'a, L>
where
    L: Logger,
{
    pub fn new(
        logger: L,
        logger_ops: &'a dyn Ops,
        http_client_ops: &'a dyn http_client::ResponseOps,
    ) -> LoggerContext<'a, L> {
        LoggerContext {
            logger,
            logger_ops,
            http_client_ops,
        }
    }

    /// Creates a new Access logger context bound to the actual Envoy ABI.
    pub fn with_default_ops(logger: L) -> Self {
        Self::new(logger, Ops::default(), http_client::ResponseOps::default())
    }
}
