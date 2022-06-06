use std::time::Duration;

use args::get_args;
use lbr::messages::{BindsInformation, CreateChannelInfo, ResponseMessage, Status};
use lbr::transport::{read_message, write_message};
use lbr::Result;

use tokio::net::TcpStream;

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();
    let (server_ip, _) = args.addr.split_once(":").expect("服务端地址错误!");
    let mut stream = TcpStream::connect(&args.addr).await?;
    let mut recv_buf = Vec::new();

    // 发送端口绑定信息
    let binds_info = BindsInformation::new(args.secret_key, args.binding)?;
    write_message(&mut stream, &binds_info).await?;
    println!("[发送绑定信息]");
    println!("{}", binds_info);

    let resp: ResponseMessage = read_message(&mut stream, &mut recv_buf).await?;
    match resp {
        ResponseMessage::Status(Status::Succeed) => {
            println!("[连接成功]: {}", &args.addr);
        }
        ResponseMessage::Status(Status::Failed { msg }) => {
            println!("[连接失败]: {}", msg);
            return Ok(());
        }
        _ => {}
    }

    // 创建隧道
    recv_buf.clear();
    let create_channels_info: Vec<CreateChannelInfo> =
        read_message(&mut stream, &mut recv_buf).await?;
    for channel_info in create_channels_info {
        let channel_addr = format!("{}:{}", &server_ip, channel_info.channel_port);
        tokio::spawn(async move {
            let timeout_job =
                tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(channel_addr))
                    .await;

            let bind_port = format!("{}:{}", channel_info.local_port, channel_info.remote_port);
            match timeout_job {
                Ok(r) => match r {
                    Ok(stream) => {
                        println!("[隧道创建成功]: {}", bind_port);
                        if let Err(err) = channel_handler(stream).await {
                            println!("[隧道断开连接]: {}", bind_port);
                            println!("ERROR: {}", err.to_string());
                        };
                    }
                    Err(err) => {
                        println!("[隧道创建失败]: {}", bind_port);
                        println!("ERROR: {}", err.to_string());
                    }
                },
                Err(_) => {
                    println!("[隧道连接超时]: {}", bind_port)
                }
            };
        });
    }

    loop {
        recv_buf.clear();
        if let Err(err) = read_message::<ResponseMessage>(&mut stream, &mut recv_buf).await {
            println!("[断开连接]: {}", &args.addr);
            println!("ERROR: {}", err.to_string());
            break;
        };
        println!("Heartbeat")
    }

    Ok(())
}

async fn channel_handler(mut stream: TcpStream) -> Result<()> {
    let mut recv_buf = Vec::new();
    loop {
        recv_buf.clear();
        let msg: &str = read_message(&mut stream, &mut recv_buf).await?;
        println!("{}", msg);
    }
}
