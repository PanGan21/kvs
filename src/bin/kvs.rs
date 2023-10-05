use std::process::exit;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Get {
        #[clap(required = true)]
        key: String,
    },
    Set {
        #[clap(required = true)]
        key: String,
        #[clap(required = true)]
        value: String,
    },
    Rm {
        #[clap(required = true)]
        key: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Get { key } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Commands::Set { key, value } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Commands::Rm { key } => {
            eprintln!("unimplemented");
            exit(1);
        }
    }
}
