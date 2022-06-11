use clap::Parser;
use config::{Args, ClientConfig};
use lbr::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let client_config = ClientConfig::new(Args::parse())?;
    let remote_addr = client_config.remote_addr()?;

    let mut handles = Vec::with_capacity(client_config.binds.len());
    for bind in client_config.binds {
        let stream = TcpStream::connect(remote_addr).await?;
        let handle = tokio::spawn(async move {
            if let Err(e) = process(stream).await {
                println!("Error: {}", e);
            };
            println!("[连接断开]: {}:{}", bind.local_port, bind.remote_port)
        });
        handles.push(handle);
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}

async fn process(mut client: TcpStream) -> Result<()> {
    Ok(())
}
