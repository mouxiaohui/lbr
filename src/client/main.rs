use std::time::Duration;

use args::get_args;
use lbr::messages::{BindsInformation, ResponseMessage};
use lbr::transport::{read_message, write_message};

use tokio::{io::AsyncWriteExt, net::TcpStream, time};

mod args;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();
    let mut stream = TcpStream::connect(&args.addr).await?;
    let mut recv_buf = [0u8; 512];

    let binds_info = BindsInformation::new(args.secret_key, args.binding)?;
    write_message(&mut stream, &binds_info).await?;
    println!("[发送绑定信息]");
    println!("{}", binds_info);

    let resp: ResponseMessage = read_message(&mut stream, &mut recv_buf).await?;
    match resp {
        ResponseMessage::Succeed => {
            println!("[连接成功]");
        }
        ResponseMessage::Failed { msg } => {
            println!("[连接失败]: {}", msg);
            return Ok(());
        }
    }

    loop {
        stream.write("Beating Heart".as_bytes()).await?;
        time::sleep(Duration::from_secs(3)).await;
    }
}
