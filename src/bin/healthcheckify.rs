use std::sync::Mutex;

use actix_web::{web::Data, App, HttpServer};
use healthcheckify::{
    config::config::AppConfig,
    healthcheck::health_checker::HealthChecker,
    http::handler::{home, service_status},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::new();
    let health_checker = HealthChecker::new(config.config_file);
    let hc = Data::new(Mutex::new(health_checker));

    println!("Listening on: {}", config.addr);

    HttpServer::new(move || {
        let app = App::new()
            .app_data(Data::clone(&hc))
            .service(home)
            .service(service_status);

        return app;
    })
    .bind(config.addr)?
    .run()
    .await
}
