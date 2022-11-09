use crate::logger::log::{log, LogLevel};

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

pub enum RequestMethod {
    POST,
    GET,
}

pub(crate) struct Node {
    pub id: String,
    config: NodeConfig,
    status: NodeStatus,
    last_check: SystemTime,
    strategy: NodeCheckStrategy,
    timeout: u64,
    method: RequestMethod,
    request_body: String,
    call_timeout: u64,
}

impl Node {
    pub fn new(
        config: NodeConfig,
        id: String,
        strategy: NodeCheckStrategy,
        timeout: u64,
        method: RequestMethod,
        request_body: Option<String>,
        call_timeout: u64,
    ) -> Self {
        let request_body = request_body.unwrap_or("".to_string());
        Self {
            id,
            config,
            status: NodeStatus::Processing,
            last_check: SystemTime::now()
                .checked_sub(Duration::from_secs(timeout + 10))
                .unwrap(),
            strategy,
            timeout,
            method,
            request_body,
            call_timeout,
        }
    }

    pub fn status(&self) -> NodeStatus {
        self.status
    }

    pub async fn check(&mut self) -> Result<NodeStatus, Box<dyn Error>> {
        log(
            format!("Checking url: '{}'", self.config.url),
            LogLevel::Info,
        );

        if self
            .last_check
            .duration_since(SystemTime::now())
            .unwrap_err()
            .duration()
            .as_secs()
            < self.timeout
        {
            // check every 10 seconds
            log("Returning cached status".to_string(), LogLevel::Info);
            return Ok(self.status());
        }

        self.status = NodeStatus::Processing;

        log(
            format!("Sending request, timeout:{}", self.call_timeout),
            LogLevel::Info,
        );
        let request = {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.call_timeout))
                .build()?;
            match self.method {
                RequestMethod::GET => client.get(&self.config.url).send().await,
                RequestMethod::POST => {
                    client
                        .post(&self.config.url)
                        .body(self.request_body.clone())
                        .send()
                        .await
                }
            }
        };

        if let Err(err) = request {
            self.status = NodeStatus::Down;
            return Err(err.into());
        }

        let response = request?;
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
                let body = response.text().await.unwrap();
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
    use super::*;

    #[tokio::test]
    async fn test_post_statuscode() {
        let node_config = NodeConfig::new("https://httpbin.org/post".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            10,
            RequestMethod::POST,
            None,
            30,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await.unwrap();
        assert_eq!(node.status, NodeStatus::Healthy);
    }

    #[tokio::test]
    async fn test_post_contains() {
        let node_config = NodeConfig::new("https://httpbin.org/post".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::BodyContains("origin".to_string()),
            10,
            RequestMethod::POST,
            None,
            30,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await.unwrap();
        assert_eq!(node.status, NodeStatus::Healthy);
    }

    #[tokio::test]
    async fn test_check_success() {
        let node_config = NodeConfig::new("https://google.com".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            10,
            RequestMethod::GET,
            None,
            30,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await.unwrap();
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
            RequestMethod::GET,
            None,
            30,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
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
            RequestMethod::GET,
            None,
            30,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Healthy);
    }

    #[tokio::test]
    async fn test_timeout() {
        let node_config = NodeConfig::new("https://httpbin.org/delay/2".to_string());
        let mut node = Node::new(
            node_config,
            "5".to_string(),
            NodeCheckStrategy::StatusCode,
            10,
            RequestMethod::GET,
            None,
            1,
        );

        assert_eq!(node.status, NodeStatus::Processing);
        let _ = node.check().await;
        assert_eq!(node.status, NodeStatus::Down);
    }
}
