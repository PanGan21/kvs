use std::{env::current_dir, process::exit};

use kvs::{KvStore, KvsError, Result};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-client")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "Get the value of a given key")]
    Get {
        #[structopt(name = "KEY", about = "String key")]
        key: String,
    },
    #[structopt(name = "set", about = "Set the value of a given key")]
    Set {
        #[structopt(name = "KEY", about = "String key")]
        key: String,
        #[structopt(name = "VALUE", about = "String value")]
        value: String,
    },
    #[structopt(name = "rm", about = "Remove a given key")]
    Remove {
        #[structopt(name = "KEY", about = "String key")]
        key: String,
    },
}

fn main() {
    let opt = Opt::from_args();
    if let Err(err) = run(opt) {
        eprintln!("{}", err);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Command::Set { key, value } => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(key, value)?
        }
        Command::Remove { key } => {
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key) {
                Ok(()) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}
