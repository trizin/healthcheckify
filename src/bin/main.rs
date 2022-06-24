use std::sync::{Arc, Mutex};

use healthchecker::{
    config::config::AppConfig,
    healthcheck::health_checker::HealthChecker,
    http::{handler::handle_connection, listener},
    thread::threadpool::ThreadPool,
};

fn main() {
    let config = AppConfig::new();

    let health_checker = HealthChecker::new(config.config_file);
    let pool = ThreadPool::new(config.thread_count).unwrap();

    let lstnr = listener::listen(&config.addr).expect("Cannot listen address");

    let hc = Arc::new(Mutex::new(health_checker));

    for stream in lstnr.incoming() {
        let stream = stream.unwrap();
        let hc_copy = Arc::clone(&hc);

        pool.execute(|| {
            if let Err(err) = handle_connection(stream, hc_copy) {
                println!("An error occured {}", err)
            }
        })
    }
}
