#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub url: String,
}

impl NodeConfig {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
