use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long, default_value = "8006")]
    pub port: String,

    #[clap(short, long)]
    pub secret_key: String,
}

pub fn get_args() -> Args {
    Args::parse()
}
