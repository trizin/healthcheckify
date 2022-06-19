use std::time::SystemTime;

use super::config::NodeConfig;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum NodeStatus {
    Healthy,
    Processing,
    Down,
}

pub(crate) struct Node {
    pub id: String,
    config: NodeConfig,
    status: NodeStatus,
    last_check: SystemTime,
}

impl Node {
    pub fn new(config: NodeConfig, id: String) -> Self {
        Self {
            id,
            config,
            status: NodeStatus::Processing,
            last_check: SystemTime::now(),
        }
    }

    async fn make_req(&self, url: &String) -> Result<u16, Box<dyn std::error::Error>> {
        let resp = reqwest::get(url).await?;
        let status_code = resp.status();
        Ok(status_code.as_u16())
    }

    pub fn status(&self) -> NodeStatus {
        self.status
    }

    pub async fn check(&mut self) {
        println!("Checking website: '{}'", self.config.url);

        if self
            .last_check
            .duration_since(SystemTime::now())
            .unwrap_err()
            .duration()
            .as_secs()
            < 10
        {
            // check every 10 seconds
            return;
        }

        self.status = NodeStatus::Processing;
        let status_code = self.make_req(&self.config.url).await;
        match status_code {
            Ok(code) => {
                if code >= 200 && code < 400 {
                    self.status = NodeStatus::Healthy;
                } else {
                    self.status = NodeStatus::Down;
                }
            }
            err => {
                println!("An error occured {}", err.unwrap_err());
                self.status = NodeStatus::Down
            }
        }
        self.last_check = SystemTime::now();
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::node::config::NodeConfig;
    use crate::healthcheck::node::model::Node;

    use super::NodeStatus;

    #[tokio::test]
    async fn test_check_success() {
        let node_config = NodeConfig::new("https://google.com".to_string(), 200);
        let mut node = Node::new(node_config, "5".to_string());

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Healthy);
    }
    #[tokio::test]
    async fn test_check_down() {
        let node_config = NodeConfig::new("https://thiswebsitedoesntexists.xcxc".to_string(), 200);
        let mut node = Node::new(node_config, "5".to_string());

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Down);
    }
    #[tokio::test]
    async fn test_check_down_status_code_mismatch() {
        let node_config = NodeConfig::new("https://google.com".to_string(), 404);
        let mut node = Node::new(node_config, "5".to_string());

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Down);
    }
}
