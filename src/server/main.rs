use lbr::network;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time,
};

const CONTROL_ADDR: &str = "0.0.0.0:8009";
const TUNNEL_ADDR: &str = "0.0.0.0:8008";
const VISIT_ADDR: &str = "0.0.0.0:8007";

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    control_channel().await?;

    Ok(())
}

// 创建控制通道，用于传递控制消息，如：心跳，创建新连接
async fn control_channel() -> Result<()> {
    let tcp_listener = network::create_tcp_listener(CONTROL_ADDR).await?;
    println!("[服务端监听]: {}", CONTROL_ADDR);

    loop {
        let (stream, socket_addr) = tcp_listener.accept().await?;
        println!("[新客户端连接]: {}", socket_addr.to_string());

        if let Err(err) = handle_stream(stream).await {
            println!("[客户端断开连接]: {}", socket_addr.to_string());
            println!("ERROR: {}", err.to_string());
            continue;
        };
    }
}

async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let bytes_read = stream.read(&mut buf).await?;

        if bytes_read == 0 {
            return Ok(());
        }

        stream.write_all(&buf[..bytes_read]).await?;
        time::sleep(time::Duration::from_secs(3)).await;
    }
}
