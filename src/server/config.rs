use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use lbr::Result;

#[derive(Parser)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long, default_value = "8006")]
    port: String,

    #[clap(short, long)]
    secret_key: String,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub server_ip: IpAddr,
    pub server_port: String,
    pub secret_key: String,
}

impl ServerConfig {
    pub fn new(args: Args) -> Result<Self> {
        Ok(Self {
            server_ip: "0.0.0.0".parse()?,
            server_port: args.port,
            secret_key: args.secret_key,
        })
    }

    pub fn server_addr(&self) -> Result<SocketAddr> {
        Ok(format!("{}:{}", self.server_ip, self.server_port).parse()?)
    }
}
