use std::env;

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn log(log: String, _log_level: LogLevel) {
    let log_level = env::var("LOG_LEVEL").unwrap_or("error".to_string());
    let log_level = log_level.to_lowercase();

    let log_level = match log_level.as_str() {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };

    if log_level as u8 <= _log_level as u8 {
        println!("{}", log);
    }
}
