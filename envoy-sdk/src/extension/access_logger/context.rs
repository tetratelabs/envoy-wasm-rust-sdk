pub struct LoggerContext<'a, L> where L: super::Logger {
    logger: L,
    ops: &'a dyn super::LogOps,
}

impl<'a, L> proxy_wasm::traits::RootContext for LoggerContext<'a, L> where L: super::Logger {
    fn on_log(&mut self) {
        self.logger.on_log(self.ops).unwrap()
    }
}

impl<'a, L> proxy_wasm::traits::Context for LoggerContext<'a, L> where L: super::Logger {}

impl<'a, L> LoggerContext<'a, L> where L: super::Logger {
    pub fn new(logger: L, ops: &'a dyn super::LogOps) -> LoggerContext<'a, L> {
        LoggerContext {
            logger: logger,
            ops: ops,
        }
    }
}
