use std::io::{Read, Write};

use regex::Regex;
use std::error::Error;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::healthcheck::health_checker::HealthChecker;
use crate::healthcheck::node::model::NodeStatus;

pub fn handle_connection(
    mut stream: TcpStream,
    health_checker: Arc<Mutex<HealthChecker>>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    Read::read(&mut stream, &mut buffer)?;

    let text = String::from_utf8_lossy(&buffer);
    let re = Regex::new("GET /(.*) HTTP")?;
    let path = re.captures(&text);
    if let Some(_path) = path {
        if _path.len() > 1 && !&_path[1].is_empty() {
            let path = &_path[1].replace('"', "");
            let stat = health_checker.lock().unwrap().check_by_id(path);

            let response = match stat {
                Ok(stat) if stat == NodeStatus::Down => get_response("error", 500),
                Ok(stat) if stat == NodeStatus::Healthy => get_response("ok", 200),
                Ok(stat) if stat == NodeStatus::Processing => get_response("ok", 200),
                Ok(_) => get_response("not found", 404),
                Err(_) => get_response("not found", 404),
            };

            Write::write(&mut stream, &response)?;
            stream.flush()?;

            return Ok(());
        } else {
            let mut response = String::from("");
            health_checker.lock().unwrap().check_all();
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
            let response = get_response(&response, 200);
            Write::write(&mut stream, &response)?;
            stream.flush()?;

            return Ok(());
        }
    }
    Err("Couldn't find the path".into())
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
