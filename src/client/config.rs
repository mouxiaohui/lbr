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
    server_addr: String,

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
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
}

impl Bind {
    pub fn local_port(&self) -> u16 {
        self.local_addr.port()
    }

    pub fn remote_port(&self) -> u16 {
        self.local_addr.port()
    }

    pub fn server_ip(&self) -> IpAddr{
        self.remote_addr.ip()
    }
}

#[derive(Debug)]
pub struct ClientConfig {
    pub server_ip: IpAddr,
    pub server_port: u16,
    pub secret_key: String,
    pub binds: Vec<Bind>,
}

impl ClientConfig {
    pub fn new(args: Args) -> Result<Self> {
        let (server_ip, server_port) = args
            .server_addr
            .split_once(":")
            .ok_or(ConfigError::ServerAddrTypeError(args.server_addr.clone()))?;
        let server_ip = server_ip.parse()?;
        let server_port = server_port.parse()?;

        let mut binds = Vec::new();
        let bs: Vec<&str> = args.binds.split(",").collect();
        for b in bs {
            let (local_port, remote_port) = b
                .split_once(":")
                .ok_or(ConfigError::BindsTypeError(args.binds.clone()))?;
            binds.push(Bind {
                local_addr: format!("127.0.0.1:{}", local_port).parse()?,
                remote_addr: SocketAddr::new(server_ip, remote_port.parse()?),
            })
        }

        Ok(Self {
            secret_key: args.secret_key,
            server_ip,
            server_port,
            binds,
        })
    }

    pub fn server_addr(&self) -> Result<SocketAddr> {
        Ok(format!("{}:{}", self.server_ip, self.server_port).parse()?)
    }
}
