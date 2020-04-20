use std::rc::Rc;

use super::config::SampleNetworkFilterConfig;
use super::filter::SampleNetworkFilter;

use envoy_sdk::extension;
use envoy_sdk::extension::Result;
use envoy_sdk::host::services::time;
use envoy_sdk::host::services::clients;

pub struct SampleNetworkFilterFactory<'a> {
    config: Rc<SampleNetworkFilterConfig>,
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,
}

impl<'a> SampleNetworkFilterFactory<'a> {
    pub fn new(time_service: &'a dyn time::Service, http_client: &'a dyn clients::http::Client) -> SampleNetworkFilterFactory<'a> {
        SampleNetworkFilterFactory{
            config: Rc::new(SampleNetworkFilterConfig::default()),
            time_service: time_service,
            http_client: http_client,
        }
    }
}

impl<'a> extension::Factory for SampleNetworkFilterFactory<'a> {
    type Extension = SampleNetworkFilter<'a>;

    fn on_configure(&mut self, _configuration_size: usize, ops: &dyn extension::factory::ConfigureOps) -> Result<bool> {
        let value = match ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => "".to_string(),
        };
        self.config = Rc::new(SampleNetworkFilterConfig::new(value));
        Ok(true)
    }

    fn new_extension(&mut self, instance_id: u32) -> Result<SampleNetworkFilter<'a>> {
        Ok(SampleNetworkFilter::new(Rc::clone(&self.config), instance_id, self.time_service, self.http_client))
    }
}
