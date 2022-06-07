use args::get_args;
use lbr::messages::{BindsInformation, CreateChannelInfo, ResponseMessage, Status};
use lbr::transport::{forward_bytes, read_message, write_message};
use lbr::Result;

use tokio::net::TcpStream;

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();
    let (server_ip, _) = args.addr.split_once(":").expect("服务端地址错误!");
    let mut stream = TcpStream::connect(&args.addr).await?;
    let mut recv_buf = vec![0u8; 1024];

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
    let create_channels_info: Vec<CreateChannelInfo> =
        read_message(&mut stream, &mut recv_buf).await?;
    println!("{:?}", create_channels_info);
    for channel_info in create_channels_info {
        let channel_addr = format!("{}:{}", &server_ip, channel_info.channel_port);
        tokio::spawn(async move {
            let res_tcp = TcpStream::connect(channel_addr).await;
            let bind_port = format!("{}:{}", channel_info.local_port, channel_info.remote_port);
            println!("[绑定成功]: {}", bind_port);
            match res_tcp {
                Ok(channel_stream) => {
                    if let Err(err) =
                        relay_to_local(&channel_info.local_port.to_string(), channel_stream).await
                    {
                        println!("[隧道断开连接]: {}", bind_port);
                        println!("ERROR: {}", err.to_string());
                    }
                }
                Err(err) => {
                    println!("[隧道创建失败]: {}", bind_port);
                    println!("ERROR: {}", err.to_string());
                }
            };
        });
    }

    loop {
        if let Err(err) = read_message::<ResponseMessage>(&mut stream, &mut recv_buf).await {
            println!("[断开连接]: {}", &args.addr);
            println!("ERROR: {}", err.to_string());
            break;
        };
        println!("Heartbeat")
    }

    Ok(())
}

async fn relay_to_local(local_port: &str, mut channel_stream: TcpStream) -> Result<()> {
    let mut buf = vec![0u8; 102400];
    let mut peek_buf = vec![0u8; 1];
    let mut local_stream = TcpStream::connect(format!("127.0.0.1:{}", local_port)).await?;

    loop {
        let peek_size = channel_stream.peek(&mut peek_buf).await?;
        if peek_size > 0 {
            println!("写入本地服务");
            forward_bytes(&mut channel_stream, &mut local_stream, &mut buf).await?;
        }
        loop {
            let peek_size = local_stream.peek(&mut peek_buf).await?;
            if peek_size > 0 {
                forward_bytes(&mut local_stream, &mut channel_stream, &mut buf).await?;
            }
        }
    }
}
