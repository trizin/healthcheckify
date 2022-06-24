use std::env;
use std::fs::read_to_string;

pub struct AppConfig {
    pub addr: String,
    pub thread_count: usize,
    pub config_file: String,
}

impl AppConfig {
    pub fn new() -> Self {
        dotenv::dotenv().ok();

        let config_file = read_to_string("./config.json").expect("Couldn't find the config file.");
        let addr = env::var("BIND_ADDR").unwrap_or_else(|_| String::from("127.0.0.1:8080"));
        let thread_count = env::var("THREAD_COUNT")
            .unwrap_or_else(|_| String::from("5"))
            .parse::<usize>()
            .unwrap();

        Self {
            addr,
            thread_count,
            config_file,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}
