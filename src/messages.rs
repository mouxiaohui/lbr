use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Status {
    Succeed,
    Failed { msg: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ResponseMessage {
    Status(Status),
    Heartbeat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BindsInformation {
    pub secret_key: String,
    pub binds: Vec<Bind>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bind {
    pub local_port: u16,
    pub remote_port: u16,
}

impl BindsInformation {
    pub fn new(secret_key: String, binding: String) -> Result<Self> {
        let mut binds: Vec<Bind> = Vec::new();

        for v in binding.split(",") {
            if let Some(local_remote) = v.split_once(":") {
                let local = local_remote.0.parse::<u16>().map_err(|err| {
                    format!("端口号格式错误: {}, {}", local_remote.0, err.to_string())
                })?;
                let remote = local_remote.1.parse::<u16>().map_err(|err| {
                    format!("端口号格式错误: {}, {}", local_remote.1, err.to_string())
                })?;

                binds.push(Bind::new(local, remote));
            };
        }

        Ok(Self { secret_key, binds })
    }
}

impl std::fmt::Display for BindsInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BindsInformation {{",)?;
        writeln!(f, "  SecretKey: {}", self.secret_key)?;
        writeln!(f, "  Bind: {{")?;
        for bind in &self.binds {
            writeln!(f, "    {{")?;
            writeln!(f, "      LocalPort: {}", bind.local_port)?;
            writeln!(f, "      RemotePort: {}", bind.remote_port)?;
            writeln!(f, "    }},")?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}

impl Bind {
    fn new(local_port: u16, remote_port: u16) -> Self {
        Self {
            local_port,
            remote_port,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateChannelInfo {
    pub local_port: u16,
    pub remote_port: u16,
    pub channel_port: u16,
}