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

use super::{ContextOps, DrainStatus, ExtensionFactory, Ops};
use crate::abi::proxy_wasm::traits::{Context, HttpContext, RootContext, StreamContext};
use crate::abi::proxy_wasm::types::ContextType;
use crate::extension::error::ErrorSink;
use crate::extension::{ConfigStatus, InstanceId};
use crate::host::ByteString;
use std::cell::RefCell;

pub(crate) enum ChildContextFactory<F>
where
    F: ExtensionFactory,
{
    StreamContextFactory(fn(&mut F, InstanceId) -> Box<dyn StreamContext>),
    HttpContextFactory(fn(&mut F, InstanceId) -> Box<dyn HttpContext>),
}

pub(crate) struct ExtensionFactoryContext<'a, F>
where
    F: ExtensionFactory,
{
    factory: RefCell<F>,
    context_ops: &'a dyn ContextOps,
    factory_ops: &'a dyn Ops,
    error_sink: &'a dyn ErrorSink,
    child_context_factory: ChildContextFactory<F>,
}

impl<'a, F> RootContext for ExtensionFactoryContext<'a, F>
where
    F: ExtensionFactory,
{
    fn on_configure(&mut self, configuration_size: usize) -> bool {
        let config = if configuration_size == 0 {
            Ok(ByteString::default())
        } else {
            self.context_ops.configuration(0, configuration_size)
        };
        match config.and_then(|config| {
            self.factory
                .borrow_mut()
                .on_configure(config, self.factory_ops.as_configure_ops())
        }) {
            Ok(status) => status.as_bool(),
            Err(err) => {
                self.error_sink
                    .observe("failed to configure extension", &err);
                ConfigStatus::Rejected.as_bool()
            }
        }
    }

    fn get_type(&self) -> Option<ContextType> {
        match self.child_context_factory {
            ChildContextFactory::HttpContextFactory(_) => Some(ContextType::HttpContext),
            ChildContextFactory::StreamContextFactory(_) => Some(ContextType::StreamContext),
        }
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        match self.child_context_factory {
            ChildContextFactory::HttpContextFactory(f) => Some(f(
                &mut self.factory.borrow_mut(),
                InstanceId::from(context_id),
            )),
            _ => None,
        }
    }

    fn create_stream_context(&self, context_id: u32) -> Option<Box<dyn StreamContext>> {
        match self.child_context_factory {
            ChildContextFactory::StreamContextFactory(f) => Some(f(
                &mut self.factory.borrow_mut(),
                InstanceId::from(context_id),
            )),
            _ => None,
        }
    }
}

impl<'a, F> Context for ExtensionFactoryContext<'a, F>
where
    F: ExtensionFactory,
{
    fn on_done(&mut self) -> bool {
        match self.factory.borrow_mut().on_drain() {
            Ok(status) => status.as_bool(),
            Err(err) => {
                self.error_sink
                    .observe("failed to initiate draining of the extension", &err);
                DrainStatus::Ongoing.as_bool()
            }
        }
    }
}

impl<'a, F> ExtensionFactoryContext<'a, F>
where
    F: ExtensionFactory,
{
    pub fn new(
        factory: F,
        context_ops: &'a dyn ContextOps,
        factory_ops: &'a dyn Ops,
        error_sink: &'a dyn ErrorSink,
        child_context_factory: ChildContextFactory<F>,
    ) -> Self {
        ExtensionFactoryContext {
            factory: RefCell::new(factory),
            context_ops,
            factory_ops,
            error_sink,
            child_context_factory,
        }
    }

    /// Creates a new factory context bound to the actual Envoy ABI.
    pub fn with_default_ops(factory: F, child_context_factory: ChildContextFactory<F>) -> Self {
        Self::new(
            factory,
            ContextOps::default(),
            Ops::default(),
            ErrorSink::default(),
            child_context_factory,
        )
    }
}
