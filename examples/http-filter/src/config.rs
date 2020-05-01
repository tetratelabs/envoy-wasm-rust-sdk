pub struct SampleHttpFilterConfig {
    pub value: String,
}

impl SampleHttpFilterConfig {
    pub fn new(value: String) -> SampleHttpFilterConfig {
        SampleHttpFilterConfig { value }
    }
}

impl Default for SampleHttpFilterConfig {
    fn default() -> Self {
        SampleHttpFilterConfig {
            value: String::new(),
        }
    }
}
