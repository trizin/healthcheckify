use std::error::Error;

use crate::healthcheck::{
    node::model::Node,
    node::model::{NodeCheckStrategy, NodeStatus, RequestMethod},
    parser::parse_config,
};

use super::node::config::NodeConfig;

pub struct HealthChecker {
    nodes: Vec<Node>,
}

impl HealthChecker {
    pub fn new(json_config: String) -> Self {
        let node_configs = parse_config(json_config).unwrap();

        let mut nodes: Vec<Node> = Vec::with_capacity(node_configs.len());
        for config in node_configs.iter() {
            let node_config = NodeConfig::new(config["url"].as_str().unwrap().trim().to_string());
            let id = config["id"].as_str().unwrap();
            let timeout = config["interval"].as_u64().unwrap_or(10u64);
            let strategy = match config["strategy"].as_str().unwrap_or("statuscode") {
                "stringcontains" => {
                    let _contains_string = config["strategy_string"]
                        .as_str()
                        .expect("Strategy search string not defined");

                    NodeCheckStrategy::BodyContains(_contains_string.to_string())
                }
                _ => NodeCheckStrategy::StatusCode, // default strategy
            };
            let lowercase_strategy = config["method"]
                .as_str()
                .unwrap_or("get")
                .to_ascii_lowercase();

            let method = match lowercase_strategy.as_str() {
                "post" => RequestMethod::POST,
                _ => RequestMethod::GET,
            };

            let request_body = config["requestBody"].as_str().unwrap_or("").to_string();
            // convert to Option
            let request_body = if request_body.is_empty() {
                None
            } else {
                Some(request_body)
            };

            let call_timeout = config["callTimeout"].as_u64().unwrap_or(30u64);

            nodes.push(Node::new(
                node_config,
                id.to_string(),
                strategy,
                timeout,
                method,
                request_body,
                call_timeout,
            ));
        }

        println!("Health checker loaded with {} nodes", nodes.len());

        Self { nodes }
    }

    pub fn get_node_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter() {
            ids.push(node.id.clone());
        }
        ids
    }

    pub fn status(&self, u: usize) -> NodeStatus {
        self.nodes[u].status()
    }

    pub fn status_by_id(&self, id: &str) -> Option<NodeStatus> {
        self.nodes
            .iter()
            .find(|&x| x.id == id)
            .map(|node| node.status())
    }

    pub async fn check(&mut self, u: usize) -> Result<NodeStatus, Box<dyn Error>> {
        self.nodes[u].check().await
    }

    pub async fn check_by_id(&mut self, id: &str) -> Result<NodeStatus, Box<dyn Error>> {
        match self.nodes.iter_mut().find(|x| x.id == id) {
            Some(x) => x.check().await,
            None => Err("Cannot find node".into()),
        }
    }

    pub async fn check_all(&mut self) {
        for node in self.nodes.iter_mut() {
            _ = node.check().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_success() {
        let data = r#"
        [
        {
            "id":"test",
            "url": "http://localhost:2461/endb"
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());
        assert_eq!(checker.nodes.len(), 1);
        assert_eq!(checker.nodes[0].id, "test");
        assert_eq!(checker.status(0), NodeStatus::Processing);

        checker.check_all().await;

        assert_eq!(checker.status(0), NodeStatus::Down);
    }

    #[tokio::test]
    async fn test_check_multiple() {
        let data = r#"
        [
        {
            "id":"test1",
            "url": "http://localhost:2461/endb"
        },
        {
            "id":"test2",
            "url": "https://google.com"
        },
        {
            "id":"test3",
            "url": "http://osdfsdfksdf.comasdas"
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());

        checker.check_all().await;

        assert_eq!(checker.status(0), NodeStatus::Down);
        assert_eq!(checker.status(1), NodeStatus::Healthy);
        assert_eq!(checker.status(2), NodeStatus::Down);

        assert_eq!(checker.status_by_id("test1").unwrap(), NodeStatus::Down);
        assert_eq!(checker.status_by_id("test2").unwrap(), NodeStatus::Healthy);
        assert_eq!(checker.status_by_id("test3").unwrap(), NodeStatus::Down);
    }

    #[tokio::test]
    async fn test_stringcontains_strategy() {
        let data = r#"
        [
        {
            "id":"test1",
            "url": "https://cheat.sh/",
            "strategy": "stringcontains",
            "strategy_string":"The only cheat sheet",
            "interval": 10
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());
        _ = checker.check_by_id("test1").await;

        assert_eq!(checker.status(0), NodeStatus::Healthy);

        assert_eq!(checker.status_by_id("test1").unwrap(), NodeStatus::Healthy);
    }
    #[tokio::test]
    async fn test_stringcontains_strategy_fails() {
        let data = r#"
        [
        {
            "id":"test1",
            "url": "https://cheat.sh/",
            "strategy": "stringcontains",
            "strategy_string":"SOME RANDOM STUFF",
            "interval": 10
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());

        _ = checker.check_by_id("test1").await;

        assert_eq!(checker.status(0), NodeStatus::Down);
        assert_eq!(checker.status_by_id("test1").unwrap(), NodeStatus::Down);
    }

    #[tokio::test]
    async fn test_post_method() {
        let data = r#"
        [
        {
            "id":"test1",
            "url": "http://httpbin.org/post",
            "method": "post",
            "interval": 10,
            "strategy": "stringcontains",
            "strategy_string":"origin"
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());

        _ = checker.check_by_id("test1").await;

        assert_eq!(checker.status(0), NodeStatus::Healthy);
    }
}
