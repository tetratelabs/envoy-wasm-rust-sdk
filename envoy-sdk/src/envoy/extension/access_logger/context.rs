pub struct LoggerContext<L, O> where L: super::Logger, O: super::LogOps {
    logger: L,
    ops: O,
}

impl<L, O> proxy_wasm::traits::RootContext for LoggerContext<L, O> where L: super::Logger, O: super::LogOps {
    fn on_log(&mut self) {
        self.logger.on_log(&self.ops).unwrap()
    }
}

impl<L, O> proxy_wasm::traits::Context for LoggerContext<L, O> where L: super::Logger, O: super::LogOps {}

impl<L, O>  LoggerContext<L, O> where L: super::Logger, O: super::LogOps {
    pub fn new(logger: L, ops: O) -> LoggerContext<L, O> {
        LoggerContext {
            logger: logger,
            ops: ops,
        }
    }
}
