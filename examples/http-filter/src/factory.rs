use std::rc::Rc;

use super::config::SampleHttpFilterConfig;
use super::filter::SampleHttpFilter;

use envoy_sdk::extension;
use envoy_sdk::extension::Result;
use envoy_sdk::host::services::clients;
use envoy_sdk::host::services::time;

pub struct SampleHttpFilterFactory<'a> {
    config: Rc<SampleHttpFilterConfig>,
    time_service: &'a dyn time::Service,
    http_client: &'a dyn clients::http::Client,
}

impl<'a> SampleHttpFilterFactory<'a> {
    pub fn new(
        time_service: &'a dyn time::Service,
        http_client: &'a dyn clients::http::Client,
    ) -> SampleHttpFilterFactory<'a> {
        SampleHttpFilterFactory {
            config: Rc::new(SampleHttpFilterConfig::default()),
            time_service,
            http_client,
        }
    }
}

impl<'a> extension::Factory for SampleHttpFilterFactory<'a> {
    type Extension = SampleHttpFilter<'a>;

    fn on_configure(
        &mut self,
        _configuration_size: usize,
        ops: &dyn extension::factory::ConfigureOps,
    ) -> Result<bool> {
        let value = match ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => String::new(),
        };
        self.config = Rc::new(SampleHttpFilterConfig::new(value));
        Ok(true)
    }

    fn new_extension(&mut self, instance_id: u32) -> Result<SampleHttpFilter<'a>> {
        Ok(SampleHttpFilter::new(
            Rc::clone(&self.config),
            instance_id,
            self.time_service,
            self.http_client,
        ))
    }
}
