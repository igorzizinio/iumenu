use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Run in server mode (daemon that stays in background)
    #[arg(long, default_value = "false")]
    pub server: bool,

    /// Toggle the launcher window (client mode)
    #[arg(long, default_value = "false")]
    pub toggle: bool,
}

pub fn parse_arguments() -> Args {
    Args::parse()
}
