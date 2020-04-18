use std::rc::Rc;

use super::config::SampleHttpFilterConfig;
use super::filter::SampleHttpFilter;

use envoy_sdk::extension;
use envoy_sdk::extension::Result;

pub struct SampleHttpFilterFactory {
    config: Rc<SampleHttpFilterConfig>,
}

impl SampleHttpFilterFactory {
    pub fn new() -> SampleHttpFilterFactory {
        SampleHttpFilterFactory{
            config: Rc::new(SampleHttpFilterConfig::default())
        }
    }
}

impl extension::Factory for SampleHttpFilterFactory {
    type Extension = SampleHttpFilter;

    fn on_configure(&mut self, _configuration_size: usize, ops: &dyn extension::factory::ConfigureOps) -> Result<bool> {
        let value = match ops.get_configuration()? {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            },
            None => "".to_string(),
        };
        self.config = Rc::new(SampleHttpFilterConfig::new(value));
        Ok(true)
    }

    fn new_extension(&mut self, context_id: u32) -> Result<SampleHttpFilter> {
        Ok(SampleHttpFilter::new(Rc::clone(&self.config), context_id))
    }
}
