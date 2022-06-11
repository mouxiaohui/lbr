use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ResponseMessage {
    Heartbeat,
    NewRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bind {
    pub secret_key: String,
    pub remote_port: String,
}

impl Bind {
    pub fn new(secret_key: String, remote_port: String) -> Self {
        Self {
            secret_key,
            remote_port,
        }
    }
}
