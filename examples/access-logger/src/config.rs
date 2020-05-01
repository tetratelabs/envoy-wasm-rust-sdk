pub struct SampleAccessLoggerConfig {
    pub value: String,
}

impl SampleAccessLoggerConfig {
    pub fn new(value: String) -> SampleAccessLoggerConfig {
        SampleAccessLoggerConfig { value }
    }
}

impl Default for SampleAccessLoggerConfig {
    fn default() -> Self {
        SampleAccessLoggerConfig {
            value: String::new(),
        }
    }
}
