use std::{
    env,
    fs::read_to_string,
    sync::{Arc, Mutex},
};

use healthchecker::{
    healthcheck::health_checker::HealthChecker,
    http::{handler::handle_connection, listener},
    thread::threadpool::ThreadPool,
};

fn main() {
    dotenv::dotenv().ok();

    let addr = env::var("BIND_ADDR").unwrap_or_else(|_| String::from("127.0.0.1:8080"));
    let thread_count = env::var("THREAD_COUNT")
        .unwrap_or_else(|_| String::from("5"))
        .parse::<usize>()
        .unwrap();

    let config_file = read_to_string("./config.json").except("Couldn't find the config file.");
    let health_checker = HealthChecker::new(config_file);
    let pool = ThreadPool::new(thread_count).unwrap();

    let lstnr = listener::listen(&addr).expect("Cannot listen address");

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
