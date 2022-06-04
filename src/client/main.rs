use std::time::Duration;

use args::get_args;
use messages::BindsInformation;
use tokio::{io::AsyncWriteExt, net::TcpStream, time};

mod args;
mod messages;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = get_args();
    let binds_info = BindsInformation::new(&args)?;
    let binds_bytes = bincode::serialize(&binds_info)?;
    let mut stream = TcpStream::connect(&args.addr).await?;

    println!("[发送绑定信息]");
    println!("{}", binds_info);
    stream.write_all(&binds_bytes).await?;

    loop {
        stream.write_all("Beating Heart".as_bytes()).await?;
        time::sleep(Duration::from_secs(3)).await;
    }
}
