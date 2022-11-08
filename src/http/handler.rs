use std::sync::{Arc, Mutex};

use actix_web::{get, http::StatusCode, web, HttpResponse, Responder};

use crate::healthcheck::{health_checker::HealthChecker, node::model::NodeStatus};

#[get("/")]
pub async fn home(health_checker: web::Data<Arc<Mutex<HealthChecker>>>) -> impl Responder {
    let mut response = String::from("");
    health_checker.lock().unwrap().check_all().await;
    let ids = health_checker.lock().unwrap().get_node_ids();
    for node_id in ids {
        response += &format!("{}: ", node_id);
        let stat = health_checker.lock().unwrap().status_by_id(&node_id);
        let answer = match stat {
            Some(stat) if stat == NodeStatus::Down => format!("{}\n", "down"),
            Some(stat) if stat == NodeStatus::Healthy => format!("{}\n", "healthy"),
            Some(stat) if stat == NodeStatus::Processing => format!("{}\n", "processing"),
            _ => format!("{}\n", "error"),
        };
        response += &answer;
    }

    get_response(&response, 200)
}

#[get("/{service_id}")]
pub async fn service_status(
    path: web::Path<String>,
    health_checker: web::Data<Arc<Mutex<HealthChecker>>>,
) -> impl Responder {
    let node_id = path.into_inner();
    let stat = health_checker
        .lock()
        .unwrap()
        .check_by_id(node_id.as_str())
        .await;

    match stat {
        Ok(stat) if stat == NodeStatus::Down => get_response("error", 500),
        Ok(stat) if stat == NodeStatus::Healthy => get_response("ok", 200),
        Ok(stat) if stat == NodeStatus::Processing => get_response("ok", 200),
        Ok(_) => get_response("not found", 404),
        Err(_) => get_response("not found", 404),
    }
}

fn get_response(message: &str, response_code: u16) -> HttpResponse {
    HttpResponse::build(StatusCode::from_u16(response_code).unwrap()).body(message.to_string())
}
