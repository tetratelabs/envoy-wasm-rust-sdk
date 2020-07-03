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

pub struct FactoryContext<'a, F>
where
    F: super::Factory,
{
    factory: F,
    factory_ops: &'a dyn super::Ops,
}

impl<'a, F> proxy_wasm::traits::RootContext for FactoryContext<'a, F>
where
    F: super::Factory,
{
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        self.factory
            .on_configure(
                plugin_configuration_size,
                self.factory_ops.as_configure_ops(),
            )
            .unwrap()
    }
}

impl<'a, F> proxy_wasm::traits::Context for FactoryContext<'a, F>
where
    F: super::Factory,
{
    fn on_done(&mut self) -> bool {
        self.factory
            .on_drain(self.factory_ops.as_done_ops())
            .unwrap()
    }
}

impl<'a, F> FactoryContext<'a, F>
where
    F: super::Factory,
{
    pub fn new(factory: F, factory_ops: &'a dyn super::Ops) -> FactoryContext<'a, F> {
        FactoryContext {
            factory,
            factory_ops,
        }
    }
}
