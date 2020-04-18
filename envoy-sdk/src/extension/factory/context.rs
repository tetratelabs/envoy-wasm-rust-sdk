pub struct FactoryContext<F, O> where F: super::Factory, O: super::Ops {
    factory: F,
    ops: O,
}

impl<F, O> proxy_wasm::traits::RootContext for FactoryContext<F, O> where F: super::Factory, O: super::Ops {
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        self.factory.on_configure(plugin_configuration_size, &self.ops).unwrap()
    }
}

impl<F, O> proxy_wasm::traits::Context for FactoryContext<F, O> where F: super::Factory, O: super::Ops {
    fn on_done(&mut self) -> bool {
        self.factory.on_drain(&self.ops).unwrap()
    }
}

impl<F, O> FactoryContext<F, O> where F: super::Factory, O: super::Ops {
    pub fn new(factory: F, ops: O) -> FactoryContext<F, O> {
        FactoryContext {
            factory: factory,
            ops: ops,
        }
    }
}
