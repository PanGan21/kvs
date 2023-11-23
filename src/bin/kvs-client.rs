use std::{net::SocketAddr, process::exit};

use kvs::{KvsClient, Result};
use structopt::{clap::AppSettings, StructOpt};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const ADDRESS_FORMAT: &str = "IP:PORT";

#[derive(StructOpt, Debug)]
#[structopt(
    name = "kvs-client",
    global_settings = &
    [AppSettings::DisableHelpSubcommand, AppSettings::VersionlessSubcommands]
)]
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
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "set", about = "Set the value of a given key")]
    Set {
        #[structopt(name = "KEY", about = "String key")]
        key: String,
        #[structopt(name = "VALUE", about = "String value")]
        value: String,
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "rm", about = "Remove a given key")]
    Remove {
        #[structopt(name = "KEY", about = "String key")]
        key: String,
        #[structopt(
            long,
            help = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    if let Err(err) = run(opt).await {
        eprintln!("{}", err);
        exit(1);
    }
}

async fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            let mut client = KvsClient::connect(addr).await?;
            if let Some(value) = client.get(key).await? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Command::Set { key, value, addr } => {
            let mut client = KvsClient::connect(addr).await?;
            client.set(key, value).await?
        }
        Command::Remove { key, addr } => {
            let mut client = KvsClient::connect(addr).await?;
            client.remove(key).await?;
        }
    }
    Ok(())
}
