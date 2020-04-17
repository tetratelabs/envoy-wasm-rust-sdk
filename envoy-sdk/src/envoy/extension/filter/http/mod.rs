extern crate std;
use std::prelude::v1::*;

pub mod ops;
pub mod context;
pub use context::FilterContext; 

use crate::envoy::host;
use crate::envoy::extension::Result;

use proxy_wasm::types::{Action, Bytes};

pub type FilterHeadersStatus = Action;
pub type FilterDataStatus = Action;
pub type FilterTrailersStatus = Action;

pub trait Filter {
    fn on_request_headers(&mut self, _num_headers: usize, _ops: &dyn RequestHeadersOps) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    fn on_request_body(&mut self, _body_size: usize, _end_of_stream: bool, _ops: &dyn RequestBodyOps) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    fn on_request_trailers(&mut self, _num_trailers: usize, _ops: &dyn RequestTrailersOps) -> Result<FilterTrailersStatus> {
        Ok(FilterTrailersStatus::Continue)
    }

    fn on_response_headers(&mut self, _num_headers: usize, _ops: &dyn ResponseHeadersOps) -> Result<FilterHeadersStatus> {
        Ok(FilterHeadersStatus::Continue)
    }

    fn on_response_body(&mut self, _body_size: usize, _end_of_stream: bool, _ops: &dyn ResponseBodyOps) -> Result<FilterDataStatus> {
        Ok(FilterDataStatus::Continue)
    }

    fn on_response_trailers(&mut self, _num_trailers: usize, _ops: &dyn ResponseTrailersOps) -> Result<FilterTrailersStatus> {
        Ok(FilterTrailersStatus::Continue)
    }

    fn on_exchange_complete(&mut self) -> Result<()> {
        Ok(())
    }
}

pub trait RequestHeadersOps: RequestFlowOps {
    fn get_request_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_request_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_request_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_request_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait RequestBodyOps: RequestFlowOps {
    fn get_request_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait RequestTrailersOps: RequestFlowOps {
    fn get_request_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_request_trailers(&self, trailers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_request_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn set_request_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_request_trailer(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait RequestFlowOps {
    fn resume_request(&self) -> host::Result<()>;

    fn clear_route_cache(&self) -> host::Result<()>;

    fn send_response(
        &self,
        status_code: u32,
        headers: Vec<(&str, &str)>,
        body: Option<&[u8]>,
    ) -> host::Result<()>;
}

pub trait ResponseHeadersOps: ResponseFlowOps {
    fn get_response_headers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_headers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_response_header(&self, name: &str) -> host::Result<Option<String>>;

    fn set_response_header(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_response_header(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait ResponseBodyOps: ResponseFlowOps {
    fn get_response_body(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait ResponseTrailersOps: ResponseFlowOps {
    fn get_response_trailers(&self) -> host::Result<Vec<(String, String)>>;

    fn set_response_trailers(&self, headers: Vec<(&str, &str)>) -> host::Result<()>;

    fn get_response_trailer(&self, name: &str) -> host::Result<Option<String>>;

    fn set_response_trailer(&self, name: &str, value: Option<&str>) -> host::Result<()>;

    fn add_response_trailer(&self, name: &str, value: &str) -> host::Result<()>;
}

pub trait ResponseFlowOps {
    fn resume_response(&self) -> host::Result<()>;
}

pub trait Ops: RequestHeadersOps + RequestBodyOps + RequestTrailersOps
 + ResponseHeadersOps + ResponseBodyOps + ResponseTrailersOps
 where Self: std::marker::Sized {}

impl<T> Ops for T 
 where T: RequestHeadersOps + RequestBodyOps + RequestTrailersOps
 + ResponseHeadersOps + ResponseBodyOps + ResponseTrailersOps {}
