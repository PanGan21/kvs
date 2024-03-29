use std::{env::current_dir, fs, net::SocketAddr, process::exit};

use kvs::{thread_pool::RayonThreadPool, KvStore, KvsEngine, KvsServer, Result, SledKvsEngine};
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
    #[derive(PartialEq, Debug, Clone, Copy)]
    #[allow(non_camel_case_types)]
    pub enum Engine {
        kvs,
        sled,
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut opt = Opt::from_args();

    let res = async {
        let initialized_engine = get_initialized_engine()?;

        if opt.engine.is_none() {
            opt.engine = initialized_engine;
        }

        if initialized_engine.is_some() && opt.engine != initialized_engine {
            error!("Wrong engine selected!");
            exit(1);
        }

        run(opt).await
    };

    if let Err(err) = res.await {
        error!("{}", err);
        exit(1);
    }
}

async fn run(opt: Opt) -> Result<()> {
    let engine = opt.engine.unwrap_or(DEFAULT_ENGINE);

    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine);
    info!("Listening on {}", opt.addr);

    // write engine to engine file
    fs::write(current_dir()?.join("engine"), format!("{}", engine))?;

    let max_threads = num_cpus::get() as u32;

    match engine {
        Engine::kvs => {
            run_with_engine(
                KvStore::<RayonThreadPool>::open(current_dir()?, max_threads)?,
                opt.addr,
            )
            .await
        }
        Engine::sled => {
            run_with_engine(
                SledKvsEngine::<RayonThreadPool>::new(sled::open(current_dir()?)?, max_threads)?,
                opt.addr,
            )
            .await
        }
    }
}

async fn run_with_engine<T: KvsEngine>(engine: T, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr).await
}

fn get_initialized_engine() -> Result<Option<Engine>> {
    let engine = current_dir()?.join("engine");
    if !engine.exists() {
        return Ok(None);
    }

    match fs::read_to_string(engine)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!("Content of engine file is invalid: {}", e);
            Ok(None)
        }
    }
}
