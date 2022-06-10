mod args;
use args::get_args;
use lbr::messages::{Bind, ResponseMessage};
use lbr::transport::{read_message, write_message};
use lbr::Result;
use tokio::io::copy;

use std::sync::Arc;
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();

    let main_addr = format!("0.0.0.0:{}", args.port);
    let main_listener = TcpListener::bind(&main_addr).await?;
    println!("[服务端运行]: {}", &main_addr);
    loop {
        let (stream, addr) = main_listener.accept().await?;
        println!("[新客户端连接]: {}", addr);
        tokio::spawn(async move {
            if let Err(err) = process(stream).await {
                println!("Error: {}", err);
            };
            println!("[客户端断开连接]: {}", addr);
        });
    }
}

async fn process(stream: TcpStream) -> Result<()> {
    let mut recv_buf = vec![0; 1024];
    let client_stream_arc = Arc::new(Mutex::new(stream));

    let mut client_guard = client_stream_arc.lock().await;
    let bind: Bind = read_message(&mut client_guard, &mut recv_buf).await?;
    drop(client_guard);

    let client_clone = client_stream_arc.clone();
    tokio::spawn(async move {
        if let Err(err) = visit_server(client_clone, &bind.remote_port).await {
            println!("Error: {}", err);
        };
    });

    loop {
        let mut client_guard = client_stream_arc.lock().await;
        write_message(&mut client_guard, &ResponseMessage::Heartbeat).await?;
        drop(client_guard);
        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn visit_server(client: Arc<Mutex<TcpStream>>, port: &str) -> Result<()> {
    // let mut buf = vec![0; 10240];
    let visit_addr = format!("0.0.0.0:{}", port);
    let visit_listener = TcpListener::bind(&visit_addr).await?;
    let channel_listener = TcpListener::bind("0.0.0.0:0").await?;
    let channel_port = channel_listener.local_addr()?.port().to_string();
    println!("[访问内网服务]: {}", visit_listener.local_addr()?);

    loop {
        let (mut visit, _) = visit_listener.accept().await?;
        let mut client_guard = client.lock().await;
        let create_channel = ResponseMessage::CreateChannel(channel_port.clone());
        write_message(&mut client_guard, &create_channel).await?;

        let (mut channel, _) = channel_listener.accept().await?;
        println!("成功创建隧道");
        if let Err(err) = copy(&mut visit, &mut channel).await {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("OK")
            }
        };
        if let Err(err) = copy(&mut channel, &mut visit).await {
            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("OK")
            }
        };
    }
}
