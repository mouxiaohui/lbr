use std::io::Result;

use tokio::net::{TcpListener, TcpStream};

pub async fn create_tcp_conn(addr: &str) -> Result<TcpStream> {
    TcpStream::connect(addr).await
}

pub async fn create_tcp_listener(addr: &str) -> Result<TcpListener> {
    TcpListener::bind(addr).await
}
