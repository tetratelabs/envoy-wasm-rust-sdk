pub struct SampleNetworkFilterConfig {
    pub value: String,
}

impl SampleNetworkFilterConfig {
    pub fn new(value: String) -> SampleNetworkFilterConfig {
        SampleNetworkFilterConfig { value: value }
    }
}

impl Default for SampleNetworkFilterConfig {
    fn default() -> Self {
        SampleNetworkFilterConfig {
            value: "".to_string(),
        }
    }
}
