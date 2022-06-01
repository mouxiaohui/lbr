use std::io;

use lbr::network;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const REMOTE_CONTROL_ADDR: &str = "127.0.0.1:8009";
const REMOTE_SERVER_ADDR: &str = "127.0.0.1:8008";

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    match network::create_tcp_conn(REMOTE_CONTROL_ADDR).await {
        Ok(mut tcp_stream) => {
            for _ in 0..10 {
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read!");

                tcp_stream.write_all(&input.as_bytes()).await?;

                let mut reader = BufReader::new(&mut tcp_stream);
                let mut buffer: Vec<u8> = Vec::new();
                reader
                    .read_until(b'\n', &mut buffer)
                    .await
                    .expect("Failed to read into buffer");
                println!(
                    "read from server: {}",
                    std::str::from_utf8(&buffer).unwrap()
                );
            }

            Ok(())
        }
        Err(err) => {
            println!("[连接失败]: {}", REMOTE_CONTROL_ADDR);
            Err(Box::from(err))
        }
    }
}
