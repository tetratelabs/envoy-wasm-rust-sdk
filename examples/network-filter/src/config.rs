pub struct SampleNetworkFilterConfig {
    pub value: String,
}

impl SampleNetworkFilterConfig {
    pub fn new(value: String) -> SampleNetworkFilterConfig {
        SampleNetworkFilterConfig { value }
    }
}

impl Default for SampleNetworkFilterConfig {
    fn default() -> Self {
        SampleNetworkFilterConfig {
            value: String::new(),
        }
    }
}
