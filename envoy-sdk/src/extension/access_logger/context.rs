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

use crate::host::services::clients;

pub struct LoggerContext<'a, L>
where
    L: super::Logger,
{
    logger: L,
    logger_ops: &'a dyn super::Ops,
    http_client_ops: &'a dyn clients::http::ResponseOps,
}

impl<'a, L> proxy_wasm::traits::RootContext for LoggerContext<'a, L>
where
    L: super::Logger,
{
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        self.logger
            .on_configure(
                plugin_configuration_size,
                self.logger_ops.as_configure_ops(),
            )
            .unwrap()
    }

    fn on_log(&mut self) {
        self.logger.on_log(self.logger_ops.as_log_ops()).unwrap();
    }
}

impl<'a, L> proxy_wasm::traits::Context for LoggerContext<'a, L>
where
    L: super::Logger,
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
                clients::http::RequestHandle::from(token_id),
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
    L: super::Logger,
{
    pub fn new(
        logger: L,
        logger_ops: &'a dyn super::Ops,
        http_client_ops: &'a dyn clients::http::ResponseOps,
    ) -> LoggerContext<'a, L> {
        LoggerContext {
            logger,
            logger_ops,
            http_client_ops,
        }
    }
}
