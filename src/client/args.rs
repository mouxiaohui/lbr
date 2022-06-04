use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long)]
    pub addr: String,

    #[clap(short, long)]
    pub secret_key: String,

    #[clap(
        short,
        long,
        help = "Remote ports are bound to local ports. Example: [7777:8080,9999:8888](local:remote)"
    )]
    pub binding: String,
}

pub fn get_args() -> Args {
    Args::parse()
}
