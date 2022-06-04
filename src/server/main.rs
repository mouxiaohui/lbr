use std::net::SocketAddr;

use args::get_args;
use lbr::{
    messages::{BindsInformation, ResponseMessage},
    transport::{read_message, write_message},
};

use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

mod args;

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
    let binds_info: BindsInformation = read_message(&mut stream, &mut recv_buf).await?;
    println!("[接收到绑定信息]: {}", addr.to_string());
    println!("{}", binds_info);

    // 判断密钥是否正确
    if secret_key != binds_info.secret_key {
        let msg = format!("[密钥错误]: {}", addr.to_string());
        write_message(&mut stream, &ResponseMessage::Failed { msg: msg.clone() }).await?;
        return Err(msg.into());
    }
    write_message(&mut stream, &ResponseMessage::Succeed).await?;

    loop {
        let size = stream.read(&mut recv_buf).await?;
        println!("{}", std::str::from_utf8(&recv_buf[0..size])?);
    }
}
