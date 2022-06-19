use std::io::{Read, Write};

use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use regex::Regex;

use crate::healthcheck::health_checker::HealthChecker;
use crate::healthcheck::node::model::NodeStatus;

pub async fn handle_connection(mut stream: TcpStream, health_checker: Arc<Mutex<HealthChecker>>) {
    let mut buffer = [0; 1024];
    Read::read(&mut stream, &mut buffer).unwrap();

    let text = String::from_utf8_lossy(&buffer);
    let re = Regex::new("GET /(.*) HTTP").unwrap();
    let path = re.captures(&text).unwrap();
    let path = &path[1].replace('"', "");

    let stat = health_checker.lock().unwrap().check_by_id(path).await;

    let response = match stat {
        Ok(stat) if stat == NodeStatus::Down => get_response("error", 500),
        Ok(stat) if stat == NodeStatus::Healthy => get_response("ok", 200),
        Ok(stat) if stat == NodeStatus::Processing => get_response("ok", 200),
        Ok(_) => get_response("not found", 404),
        Err(_) => get_response("not found", 404),
    };

    Write::write(&mut stream, &response).unwrap();
    stream.flush().unwrap();
}

fn get_response(message: &str, response_code: u32) -> Vec<u8> {
    let bytes = format!(
        "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
        response_code,
        message.len(),
        message
    );
    return bytes.as_bytes().to_vec();
}
