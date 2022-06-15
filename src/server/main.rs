use std::sync::Arc;

use clap::Parser;
use config::{Args, ServerConfig};
use err::ConnectError;
use lbr::{
    messages::{BindMessage, HEARTBEAT},
    transport::read_message,
    Result,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

mod config;
mod err;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::new(Args::parse())?;
    let listener = TcpListener::bind(config.server_addr()?).await?;

    println!("[服务端启动]: {}", config.server_port);
    while let Ok((stream, client_addr)) = listener.accept().await {
        println!("[新客户端]: {}", client_addr);
        if let Err(e) = process(stream, &config.secret_key).await {
            println!("Error: {}", e);
        };
        println!("[客户端断开连接]: {}", client_addr);
    }

    Ok(())
}

async fn process(client: TcpStream, secret_key: &str) -> Result<()> {
    let client_addr = client.peer_addr()?;
    let client = Arc::new(Mutex::new(client));
    let mut recv_buf = vec![0; 1024];

    let bind_msg = {
        let mut client_guard = client.lock().await;
        let bind_msg: BindMessage = read_message(&mut client_guard, &mut recv_buf).await?;
        bind_msg
    };

    if bind_msg.secret_key != secret_key {
        return Err(ConnectError::VerificationFailed.into());
    }

    let f2 = async move {
        loop {
            let mut client_guard = client.lock().await;
            match client_guard.read_u8().await {
                Ok(p) => {
                    if p != HEARTBEAT {
                        break;
                    }
                }
                Err(_) => break,
            };
        }

        ConnectError::ClientDisconnected(client_addr.to_string())
    };

    let res = tokio::select! {
        res = f2 => res
    };

    Err(res.into())
}
