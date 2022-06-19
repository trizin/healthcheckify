use std::net::TcpListener;
pub fn listen(addr: &str) -> Result<TcpListener, std::io::Error> {
    println!("Listening on {}", addr);
    TcpListener::bind(addr)
}
