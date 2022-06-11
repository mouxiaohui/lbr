use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use lbr::Result;

#[derive(Debug)]
pub enum ConfigError {
    ServerAddrTypeError(String),
    BindsTypeError(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ServerAddrTypeError(addr) => write!(f, "服务端地址格式错误: [{}]", &addr),
            ConfigError::BindsTypeError(binds) => write!(f, "端口绑定格式错误: [{}]", &binds),
        }
    }
}

#[derive(Parser)]
#[clap(author, version)]
pub struct Args {
    #[clap(short = 'i', long)]
    remote_addr: String,

    #[clap(short, long)]
    secret_key: String,

    #[clap(
        short,
        long,
        help = "本地与远程端口绑定 例如: 7777:8080,9999:8888(local:remote)如有多个用逗号隔开"
    )]
    binds: String,
}

#[derive(Debug)]
pub struct Bind {
    pub local_port: String,
    pub remote_port: String,
}

#[derive(Debug)]
pub struct ClientConfig {
    pub remote_ip: IpAddr,
    pub remote_port: String,
    pub secret_key: String,
    pub binds: Vec<Bind>,
}

impl ClientConfig {
    pub fn new(args: Args) -> Result<Self> {
        let (remote_ip, remote_port) = args
            .remote_addr
            .split_once(":")
            .ok_or(ConfigError::ServerAddrTypeError(args.remote_addr.clone()))?;

        let bs: Vec<&str> = args.binds.split(",").collect();
        let mut binds = Vec::new();
        for b in bs {
            let (local_port, remote_port) = b
                .split_once(":")
                .ok_or(ConfigError::BindsTypeError(args.binds.clone()))?;
            binds.push(Bind {
                local_port: local_port.to_string(),
                remote_port: remote_port.to_string(),
            })
        }

        Ok(Self {
            secret_key: args.secret_key,
            remote_ip: remote_ip.parse()?,
            remote_port: remote_port.to_string(),
            binds,
        })
    }

    pub fn remote_addr(&self) -> Result<SocketAddr> {
        Ok(format!("{}:{}", self.remote_ip, self.remote_port).parse()?)
    }
}
