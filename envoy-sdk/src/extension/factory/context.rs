pub struct FactoryContext<'a, F> where F: super::Factory {
    factory: F,
    factory_ops: &'a dyn super::Ops,
}

impl<'a, F> proxy_wasm::traits::RootContext for FactoryContext<'a, F> where F: super::Factory {
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        self.factory.on_configure(plugin_configuration_size, self.factory_ops.as_configure_ops()).unwrap()
    }
}

impl<'a, F> proxy_wasm::traits::Context for FactoryContext<'a, F> where F: super::Factory {
    fn on_done(&mut self) -> bool {
        self.factory.on_drain(self.factory_ops.as_done_ops()).unwrap()
    }
}

impl<'a, F> FactoryContext<'a, F> where F: super::Factory {
    pub fn new(factory: F, factory_ops: &'a dyn super::Ops) -> FactoryContext<'a, F> {
        FactoryContext {
            factory: factory,
            factory_ops: factory_ops,
        }
    }
}
