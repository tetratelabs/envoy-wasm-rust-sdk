extern crate std;

pub mod ops;
pub mod context;
pub use context::FilterContext; 

use crate::envoy::host;
use crate::envoy::extension::Result;

use proxy_wasm::types::{Action, Bytes, PeerType};

pub type FilterStatus = Action;

pub trait Filter {
    fn on_new_connection(&mut self) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    fn on_downstream_data(&mut self, _data_size: usize, _end_of_stream: bool, _ops: &dyn DownstreamDataOps) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    fn on_downstream_close(&mut self, _peer_type: PeerType) -> Result<()> {
        Ok(())
    }

    fn on_upstream_data(&mut self, _data_size: usize, _end_of_stream: bool, _ops: &dyn UpstreamDataOps) -> Result<FilterStatus> {
        Ok(FilterStatus::Continue)
    }

    fn on_upstream_close(&mut self, _peer_type: PeerType) -> Result<()> {
        Ok(())
    }

    fn on_connection_complete(&mut self) -> Result<()> {
        Ok(())
    }
}

pub trait DownstreamDataOps {
    fn get_downstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait UpstreamDataOps {
    fn get_upstream_data(&self, start: usize, max_size: usize) -> host::Result<Option<Bytes>>;
}

pub trait Ops: DownstreamDataOps + UpstreamDataOps where Self: std::marker::Sized {}

impl<T> Ops for T where T: DownstreamDataOps + UpstreamDataOps {}
