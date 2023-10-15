use std::{net::SocketAddr, process::exit};

use kvs::Result;
use log::{error, info, warn, LevelFilter};
use structopt::{clap::arg_enum, StructOpt};

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const ADDRESS_FORMAT: &str = "IP:PORT";
const DEFAULT_ENGINE: Engine = Engine::kvs;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-server")]
struct Opt {
    #[structopt(
        long,
        help = "Sets the listening address",
        value_name = ADDRESS_FORMAT,
        default_value = DEFAULT_LISTENING_ADDRESS,
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[structopt(
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE_NAME",
        possible_values = &Engine::variants()
    )]
    engine: Option<Engine>,
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Engine {
        kvs,
        sled,
    }
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let opt = Opt::from_args();
    if let Err(err) = run(opt) {
        eprintln!("{}", err);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    let engine = opt.engine.unwrap_or(DEFAULT_ENGINE);

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine);
    info!("Listening on {}", opt.addr);

    Ok(())
}
