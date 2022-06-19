use serde_json::{Result, Value};

pub(crate) fn parse_config(config_str: String) -> Result<Vec<Value>> {
    let configs: Vec<Value> = serde_json::from_str(&config_str)?;

    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::parse_config;

    #[test]
    fn test_parser() {
        let data = r#"
        [
        {
            "path": "http://localhost:2461/endb"
        },
        {
            "path": "https://google.com/"
        },
        {
            "path": "http://osdfsdfksdf.comasdas"
        }
        ]"#
        .to_string();

        let parsed = parse_config(data).unwrap();

        assert!(parsed[0]["path"].as_str().unwrap() == "http://localhost:2461/endb");
        assert!(parsed[1]["path"].as_str().unwrap() == "https://google.com/");
        assert!(parsed[2]["path"].as_str().unwrap() == "http://osdfsdfksdf.comasdas");
    }
}
