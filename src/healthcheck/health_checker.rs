use std::time::SystemTime;

use crate::healthcheck::{node::model::Node, node::model::NodeStatus, parser::parse_config};

use super::node::config::NodeConfig;

pub struct HealthChecker {
    nodes: Vec<Node>,
}

impl HealthChecker {
    pub fn new(json_config: String) -> Self {
        let node_configs = parse_config(json_config).unwrap();

        let mut nodes: Vec<Node> = Vec::with_capacity(node_configs.len());
        for config in node_configs.iter() {
            let node_config = NodeConfig::new(config["path"].as_str().unwrap().trim().to_string());
            let id = config["id"].as_str().unwrap();

            nodes.push(Node::new(node_config, id.to_string()));
        }

        println!("Health checker loaded with {} nodes", nodes.len());

        Self { nodes }
    }

    pub fn status(&self, u: usize) -> NodeStatus {
        self.nodes[u].status()
    }

    pub fn status_by_id(&self, id: &str) -> Option<NodeStatus> {
        if let Some(node) = self.nodes.iter().find(|&x| x.id == id) {
            return Some(node.status());
        } else {
            return None;
        }
    }

    pub async fn check(&mut self, u: usize) {
        self.nodes[u].check().await;
    }

    pub async fn check_all(&mut self) {
        for node in &mut self.nodes {
            node.check().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::node::model::NodeStatus;

    use super::HealthChecker;

    #[tokio::test]
    async fn test_check_success() {
        let data = r#"
        [
        {
            "id":"test",
            "path": "http://localhost:2461/endb",
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
            "path": "http://localhost:2461/endb",
        },
        {
            "id":"test2",
            "path": "https://google.com",
        },
        {
            "id":"test3",
            "path": "http://osdfsdfksdf.comasdas",
        }
        ]"#;

        let mut checker = HealthChecker::new(data.to_string());

        checker.check_all().await;

        assert_eq!(checker.status(0), NodeStatus::Down);
        assert_eq!(checker.status(1), NodeStatus::Healthy);
        assert_eq!(checker.status(2), NodeStatus::Down);

        assert_eq!(checker.status_by_id("test1"), NodeStatus::Down);
        assert_eq!(checker.status_by_id("test2"), NodeStatus::Healthy);
        assert_eq!(checker.status_by_id("test3"), NodeStatus::Down);
    }
}
