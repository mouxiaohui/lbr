mod args;

use std::sync::Arc;

use args::get_args;
use lbr::messages::{Bind, ResponseMessage};
use lbr::transport::{read_message, write_message};
use lbr::Result;

use tokio::io::copy;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
        let server = Arc::new(Mutex::new(stream));

        let local_port = lr.0.clone().to_string();
        let remote_port = lr.1.clone().to_string();
        let secret_key = args.secret_key.clone();

        let handle = tokio::spawn(async move {
            if let Err(err) = process(server, &secret_key, &local_port, &remote_port).await {
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
    server_arc: Arc<Mutex<TcpStream>>,
    secret_key: &str,
    local_port: &str,
    remote_port: &str,
) -> Result<()> {
    let local_stream = TcpStream::connect(format!("127.0.0.1:{}", local_port)).await?;
    let local_arc = Arc::new(Mutex::new(local_stream));
    let mut server = server_arc.lock().await;

    let bind = Bind::new(secret_key.to_string(), remote_port.to_string());
    write_message(&mut server, &bind).await?;

    let mut buf = vec![0; 102400];
    loop {
        let resp_msg: ResponseMessage = read_message(&mut server, &mut buf).await?;
        match resp_msg {
            ResponseMessage::Heartbeat => println!("Heartbeat"),
            ResponseMessage::NewRequest => {
                let local_clone = local_arc.clone();
                let server_clone = server_arc.clone();
                tokio::spawn(async move {
                    let mut local = local_clone.lock().await;
                    let mut server = server_clone.lock().await;

                    let res = async move {
                        let (mut local_rx, mut local_tx) = local.split();
                        let (mut server_rx, mut server_tx) = server.split();
                        let f1 = copy(&mut local_rx, &mut server_tx);
                        let f2 = copy(&mut server_rx, &mut local_tx);
                        tokio::select! {
                            res = f1 => res,
                            res = f2 => res,
                        }
                    };

                    if let Err(e) = res.await {
                        println!("Error: {}", e);
                    }
                });
            }
        }
    }
}
