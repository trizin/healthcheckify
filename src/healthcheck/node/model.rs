use super::config::NodeConfig;
use std::error::Error;
use std::time::{Duration, SystemTime};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum NodeStatus {
    Healthy,
    Processing,
    Down,
}

pub enum NodeCheckStrategy {
    BodyContains(String),
    StatusCode,
}

pub(crate) struct Node {
    pub id: String,
    config: NodeConfig,
    status: NodeStatus,
    last_check: SystemTime,
    strategy: NodeCheckStrategy,
    timeout: u32,
}

impl Node {
    pub fn new(config: NodeConfig, id: String, strategy: NodeCheckStrategy, timeout: u32) -> Self {
        Self {
            id,
            config,
            status: NodeStatus::Processing,
            last_check: SystemTime::now()
                .checked_sub(Duration::from_secs(timeout as u64 + 10))
                .unwrap(),
            strategy,
            timeout,
        }
    }

    async fn make_req(
        &self,
        url: &String,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        Ok(reqwest::get(url).await?)
    }

    pub fn status(&self) -> NodeStatus {
        self.status
    }

    pub async fn check(&mut self) -> Result<NodeStatus, Box<dyn Error>> {
        println!("Checking url: '{}'", self.config.url);

        if self
            .last_check
            .duration_since(SystemTime::now())
            .unwrap_err()
            .duration()
            .as_secs()
            < self.timeout.into()
        {
            // check every 10 seconds
            return Err("Using cached value".into());
        }

        self.status = NodeStatus::Processing;
        let request = self.make_req(&self.config.url).await;

        if let Err(err) = request {
            self.status = NodeStatus::Down;
            return Err(err);
        }

        let response = request.unwrap();

        match &self.strategy {
            NodeCheckStrategy::StatusCode => {
                let status_code = response.status();
                if (200..400).contains(&status_code.as_u16()) {
                    self.status = NodeStatus::Healthy;
                } else {
                    self.status = NodeStatus::Down;
                }
            }
            NodeCheckStrategy::BodyContains(x) => {
                let body = response.text().await?;
                if body.contains(x) {
                    self.status = NodeStatus::Healthy;
                } else {
                    self.status = NodeStatus::Down;
                }
            }
        }

        self.last_check = SystemTime::now();

        Ok(self.status())
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::node::config::NodeConfig;
    use crate::healthcheck::node::model::Node;
    use crate::healthcheck::node::model::NodeCheckStrategy;

    use super::NodeStatus;

    #[tokio::test]
    async fn test_check_success() {
        let node_config = NodeConfig::new("https://google.com".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            10,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Healthy);
    }
    #[tokio::test]
    async fn test_check_down() {
        let node_config = NodeConfig::new("https://thiswebsitedoesntexists.xcxc".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            10,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let val = node.check().await;
        println!("{:?}", val);
        assert_eq!(node.status, NodeStatus::Down);
    }
    #[tokio::test]
    async fn test_check_with_high_timeout() {
        let node_config = NodeConfig::new("https://google.com".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            100000,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Healthy);
    }
}
