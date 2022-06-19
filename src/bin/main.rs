use std::{
    fs::read_to_string,
    sync::{Arc, Mutex},
};

use healthchecker::{
    healthcheck::health_checker::HealthChecker,
    http::{handler::handle_connection, listener},
    thread::threadpool::ThreadPool,
};

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8080";
    let config_file = read_to_string("./config.json").unwrap();
    let health_checker = HealthChecker::new(config_file);
    let pool = ThreadPool::new(5).unwrap();
    let lstnr = listener::listen(addr).expect("Cannot listen address");

    let hc = Arc::new(Mutex::new(health_checker));

    for stream in lstnr.incoming() {
        let stream = stream.unwrap();
        let hc_copy = Arc::clone(&hc);

        pool.execute(|| {
            handle_connection(stream, hc_copy);
        })
    }
}
