mod args;
use args::get_args;
use lbr::messages::{Bind, ResponseMessage};
use lbr::transport::{read_message, write_message};
use lbr::Result;

use tokio::io::copy;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();
    let lrs: Vec<(&str, &str)> = args
        .binding
        .split(",")
        .map(|s| s.split_once(":").expect("绑定端口格式错误!"))
        .collect();

    let mut handles = Vec::with_capacity(lrs.len());
    for lr in lrs {
        let stream = TcpStream::connect(&args.addr).await?;
        let local_port = lr.0.clone().to_string();
        let remote_port = lr.1.clone().to_string();
        let secret_key = args.secret_key.clone();
        let server_addr = args.addr.clone();

        let handle = tokio::spawn(async move {
            if let Err(err) =
                process(stream, &secret_key, &server_addr, &local_port, &remote_port).await
            {
                println!("Error: {}", err);
            };
            println!("[连接断开]: {}:{}", local_port, remote_port);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?
    }

    Ok(())
}

async fn process(
    mut server: TcpStream,
    secret_key: &str,
    server_addr: &str,
    local_port: &str,
    remote_port: &str,
) -> Result<()> {
    let (server_ip, _) = server_addr.split_once(":").expect("服务端地址错误");
    let mut local = TcpStream::connect(format!("127.0.0.1:{}", local_port)).await?;
    let bind = Bind::new(secret_key.to_string(), remote_port.to_string());
    write_message(&mut server, &bind).await?;

    let mut buf = vec![0; 102400];
    loop {
        let resp_msg: ResponseMessage = read_message(&mut server, &mut buf).await?;
        match resp_msg {
            ResponseMessage::Heartbeat => println!("Heartbeat"),
            ResponseMessage::CreateChannel(port) => {
                let mut channel = TcpStream::connect(format!("{}:{}", server_ip, port)).await?;
                if let Err(err) = copy(&mut channel, &mut local).await {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("OK")
                    }
                };
                if let Err(err) = copy(&mut local, &mut channel).await {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("OK")
                    }
                };
            }
        }
    }
}
