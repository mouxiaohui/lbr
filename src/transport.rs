use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::Result;

pub async fn write_message<T: Serialize>(stream: &mut TcpStream, message: &T) -> Result<()> {
    let resp_msg_bytes = bincode::serialize(message)?;
    stream.write(&resp_msg_bytes).await?;

    Ok(())
}

pub async fn read_message<'a, T: Deserialize<'a>>(
    stream: &mut TcpStream,
    recv_buf: &'a mut Vec<u8>,
) -> Result<T> {
    let size = stream.read(recv_buf).await?;
    let resp = bincode::deserialize(&recv_buf[..size])?;

    Ok(resp)
}

pub async fn forward_bytes(
    origin: &mut TcpStream,
    target: &mut TcpStream,
    buf: &mut Vec<u8>,
) -> Result<()> {
    loop {
        let size = origin.read(buf).await?;
        if size == 0 {
            return Ok(());
        }
        target.write(&buf[..size]).await?;
    }
}
