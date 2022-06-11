use clap::Parser;
use config::{Args, ServerConfig};
use lbr::Result;
use tokio::net::{TcpListener, TcpStream};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::new(Args::parse())?;
    let listener = TcpListener::bind(config.server_addr()?).await?;

    while let Ok((stream, client_addr)) = listener.accept().await {
        println!("[新客户端]: {}", client_addr);
        if let Err(e) = process(stream).await {
            println!("{} Error: {}", client_addr, e);
        };
        println!("[客户端断开连接]: {}", client_addr);
    }

    Ok(())
}

async fn process(client: TcpStream) -> Result<()> {
    Ok(())
}
