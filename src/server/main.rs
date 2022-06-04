use std::net::SocketAddr;

use args::get_args;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::messages::BindsInformation;

mod args;
mod messages;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();

    let server_addr = "0.0.0.0:".to_string() + &args.port;
    let listener = TcpListener::bind(&server_addr).await?;
    println!("[监听端口]: {}", server_addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("[新客户端连接]: {}", addr.to_string());

        let secret_key = args.secret_key.clone();
        tokio::spawn(async move {
            if let Err(err) = process(stream, addr, secret_key).await {
                println!("ERROR: {}", err.to_string());
            };
            println!("[客户端断开连接]: {}", addr.to_string());
        });
    }
}

async fn process(mut stream: TcpStream, addr: SocketAddr, secret_key: String) -> Result<()> {
    let mut recv_buf = [0u8; 512];

    // 接收需要绑定端口信息和密钥
    let size = stream.read(&mut recv_buf).await?;
    let binds_info: BindsInformation = bincode::deserialize(&recv_buf[0..size])?;
    println!("[接收到绑定信息]");
    println!("{}", binds_info);

    if secret_key != binds_info.secret_key {
        stream.shutdown().await?;
        return Err("密钥错误".into());
    }

    loop {
        let size = stream.read(&mut recv_buf).await?;
        println!("{}", std::str::from_utf8(&recv_buf[0..size])?);
    }
}
