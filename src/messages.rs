use serde::{Deserialize, Serialize};

pub const HEARTBEAT: u8 = 1;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BindMessage {
    pub secret_key: String,
    pub server_ip: String,
    pub remote_port: u16,
}

impl BindMessage {
    pub fn new(secret_key: String, server_ip: String, remote_port: u16) -> Self {
        Self {
            secret_key,
            server_ip,
            remote_port,
        }
    }
}
