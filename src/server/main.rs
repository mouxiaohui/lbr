use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use crate::args::get_args;
use lbr::messages::{BindsInformation, CreateChannelInfo, ResponseMessage, Status};
use lbr::transport::{read_message, write_message};
use lbr::Result;

use tokio::net::{TcpListener, TcpStream};
use tokio::time;


mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();

    let server_addr = "0.0.0.0:".to_string() + &args.port;
    let listener = TcpListener::bind(&server_addr).await?;
    println!("[监听端口]: {}", server_addr);

    loop {
        let (client_stream, addr) = listener.accept().await?;
        println!("[新客户端连接]: {}", addr.to_string());

        let secret_key = args.secret_key.clone();
        tokio::spawn(async move {
            if let Err(err) = process(client_stream, addr, secret_key).await {
                println!("ERROR: {}", err.to_string());
            };
            println!("[客户端断开连接]: {}", addr.to_string());
        });
    }
}

async fn process(mut client_stream: TcpStream, addr: SocketAddr, secret_key: String) -> Result<()> {
    let mut recv_buf = Vec::new();

    // 接收需要绑定端口信息和密钥
    let binds_info: BindsInformation = read_message(&mut client_stream, &mut recv_buf).await?;
    recv_buf.clear();
    println!("[接收到绑定信息]: {}", addr.to_string());
    println!("{}", binds_info);

    // 判断密钥是否正确
    if secret_key != binds_info.secret_key {
        let msg = format!("[密钥错误]: {}", addr.to_string());
        write_message(
            &mut client_stream,
            &ResponseMessage::Status(Status::Failed { msg: msg.clone() }),
        )
        .await?;
        return Err(msg.into());
    }
    write_message(
        &mut client_stream,
        &ResponseMessage::Status(Status::Succeed),
    )
    .await?;

    // 创建隧道, 服务端需要将用户请求转发到对应的内网服务
    let mut channels_listener: HashMap<String, TcpListener> = HashMap::new();
    let mut create_channels_info = Vec::new();
    for bind in binds_info.binds {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let channel_info = CreateChannelInfo {
            channel_port: listener.local_addr()?.port(),
            local_port: bind.local_port,
            remote_port: bind.remote_port,
        };
        create_channels_info.push(channel_info);
        channels_listener.insert(
            format!("{}:{}", bind.local_port, bind.remote_port),
            listener,
        );
    }
    let mut jhs = Vec::with_capacity(channels_listener.len());
    for (bind_port, listener) in channels_listener {
        let jh = tokio::spawn(async move {
            if let Err(err) = create_channels(&bind_port, listener).await {
                println!("[创建隧道失败]: {}", &bind_port);
                println!("ERROR: {}", err.to_string());
            };
        });
        jhs.push(jh);
    }
    // 发送创建隧道信息
    write_message(&mut client_stream, &create_channels_info).await?;

    loop {
        // 保持心跳连接
        if let Err(err) = write_message(&mut client_stream, &ResponseMessage::Heartbeat).await {
            jhs.iter().for_each(|jh| jh.abort());
            return Err(err);
        };
        time::sleep(Duration::from_secs(3)).await;
    }
}

async fn create_channels(bind_port: &str, listener: TcpListener) -> Result<()> {
    let (mut stream, _) = listener.accept().await?;
    println!("[隧道创建成功]: {}", bind_port);
    loop {
        write_message(&mut stream, &format!("隧道消息: {}", bind_port)).await?;
        // stream.write(format!("隧道消息: {}", bind_port).as_bytes()).await?;
        time::sleep(Duration::from_secs(3)).await;
    }
}
