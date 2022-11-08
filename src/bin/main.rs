use std::sync::{Arc, Mutex};

use actix_web::{web::Data, App, HttpServer};
use healthchecker::{
    config::config::AppConfig,
    healthcheck::health_checker::HealthChecker,
    http::handler::{home, service_status},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::new();
    let health_checker = HealthChecker::new(config.config_file);
    let hc = Arc::new(Mutex::new(health_checker));

    HttpServer::new(move || {
        let app = App::new()
            .app_data(Data::new(hc.clone()))
            .service(home)
            .service(service_status);

        return app;
    })
    .bind(("127.0.0.1", 4242))?
    .run()
    .await
}
