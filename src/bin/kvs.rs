use std::env::current_dir;

use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Get { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Commands::Set { key, value } => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key, value)?
        }
        Commands::Rm { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            store.remove(key)?
        }
        _ => unreachable!(),
    }
    Ok(())
}
