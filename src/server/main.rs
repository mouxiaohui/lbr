use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use crate::args::get_args;
use lbr::messages::{BindsInformation, CreateChannelInfo, ResponseMessage, Status};
use lbr::transport::{forward_bytes, read_message, write_message};
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
    let mut recv_buf = vec![0u8; 1024];

    // 接收需要绑定端口信息和密钥
    let binds_info: BindsInformation = read_message(&mut client_stream, &mut recv_buf).await?;
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

    // 创建隧道时发送给客户端的数据
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

    // 创建隧道, 服务端需要将用户请求转发到对应的内网服务
    let mut jhs = Vec::with_capacity(channels_listener.len());
    for (bind_port, listener) in channels_listener {
        let jh = tokio::spawn(async move {
            if let Err(err) = relay_to_channel(&bind_port, listener).await {
                println!("[绑定失败]: {}", &bind_port);
                println!("ERROR: {}", err.to_string());
            };
        });
        jhs.push(jh);
    }

    // 向客户端发送创建隧道信息
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

async fn relay_to_channel(bind_port: &str, listener: TcpListener) -> Result<()> {
    let (_, remote_port) = bind_port.split_once(":").unwrap();
    let (mut channel_stream, _) = listener.accept().await?;

    let user_listener = TcpListener::bind(format!("0.0.0.0:{}", remote_port)).await?;
    println!("[绑定成功]: {}", bind_port);
    let mut buf = vec![0u8; 102400];
    loop {
        let (mut user_stream, addr) = user_listener.accept().await?;
        println!("监听到: {}", addr);
        forward_bytes(&mut user_stream, &mut channel_stream, &mut buf).await?;
        let mut peek_buf = vec![0u8; 1];

        loop {
            let peek_size = channel_stream.peek(&mut peek_buf).await?;
            if peek_size > 0 {
                forward_bytes(&mut channel_stream, &mut user_stream, &mut buf).await?;
            }
        }
    }
}
