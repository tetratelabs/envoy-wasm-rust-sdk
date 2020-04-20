extern crate std;
use std::prelude::v1::*;

pub mod ops;
pub mod context;
pub use context::LoggerContext; 

use crate::host;
use crate::host::services::clients;
use crate::extension::Result;

use proxy_wasm::types::Bytes;

pub trait Logger {
    fn on_configure(&mut self, _configuration_size: usize, _logger_ops: &dyn ConfigureOps) -> Result<bool> {
        Ok(true)
    }

    fn on_log(&mut self, _logger_ops: &dyn LogOps) -> Result<()> {
        Ok(())
    }

    // Http Client callbacks

    fn on_http_call_response(&mut self, _request: clients::http::RequestHandle,
        _num_headers: usize, _body_size: usize, _num_trailers: usize,
        _http_client_ops: &dyn clients::http::ResponseOps,
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

impl<T> Ops for T where T: ConfigureOps + LogOps {
    fn as_configure_ops(&self) -> &dyn ConfigureOps { self }

    fn as_log_ops(&self) -> &dyn LogOps { self }
}
