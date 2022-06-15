use std::{sync::Arc, time::Duration};

use clap::Parser;
use config::{Args, Bind, ClientConfig};
use lbr::{
    messages::{BindMessage, HEARTBEAT},
    transport::write_message,
    Result,
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let client_config = ClientConfig::new(Args::parse())?;
    if client_config.binds.len() == 0 {
        println!("[未绑定本地与远程的端口号]");
        return Ok(());
    }

    let server_addr = client_config.server_addr()?;

    let mut handles = Vec::with_capacity(client_config.binds.len());
    for bind in client_config.binds {
        let stream = TcpStream::connect(&server_addr).await?;
        let secret_key = client_config.secret_key.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = process(stream, &bind, secret_key).await {
                println!("Error: {}", e);
            };
            println!("[连接断开]: {}:{}", bind.local_port(), bind.remote_port())
        });
        handles.push(handle);
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}

async fn process(client: TcpStream, bind: &Bind, secret_key: String) -> Result<()> {
    let client = Arc::new(Mutex::new(client));

    {
        let bind_msg = BindMessage::new(secret_key, bind.server_ip().to_string(), bind.remote_port());
        let mut client_guard = client.lock().await;
        write_message(&mut client_guard, &bind_msg).await?;
        println!("[以发送绑定信息]")
    }

    let f2 = async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let mut client_guard = client.lock().await;
            if let Err(e) = client_guard.write_u8(HEARTBEAT).await {
                return e;
            };
        }
    };

    let res = tokio::select! {
        res = f2 => res,
    };

    Err(res.into())
}
