use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use log::{debug, error};
use serde_json::Deserializer;

use crate::{
    thread_pool::ThreadPool, GetResponse, KvsEngine, RemoveResponse, Request, Result, SetResponse,
};

/// The server of the key value store.
pub struct KvsServer<T: KvsEngine, P: ThreadPool> {
    engine: T,
    pool: P,
}

impl<T: KvsEngine, P: ThreadPool> KvsServer<T, P> {
    /// Create a `KvsServer` with a given storage engine.
    pub fn new(engine: T, pool: P) -> Self {
        KvsServer { engine, pool }
    }

    /// Run the server listening on the given address
    pub fn run(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let engine = self.engine.clone();
            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = serve(engine, stream) {
                        error!("Error on serving client: {}", e);
                    }
                }
                Err(e) => error!("Connection failed: {}", e),
            })
        }
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> Result<()> {
    let peer_addr = tcp.peer_addr()?;
    let reader = BufReader::new(&tcp);
    let mut writer = BufWriter::new(&tcp);
    let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

    for req in req_reader {
        let request = req?;
        debug!("Receive request from {}: {:?}", peer_addr, request);
        match request {
            Request::Get { key } => {
                let res = match engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(err) => GetResponse::Err(format!("{}", err)),
                };

                serde_json::to_writer(&mut writer, &res)?;
                writer.flush()?;
                debug!("Response sent to {}: {:?}", peer_addr, res);
            }
            Request::Set { key, value } => {
                let res = match engine.set(key, value) {
                    Ok(_) => SetResponse::Ok(()),
                    Err(err) => SetResponse::Err(format!("{}", err)),
                };

                serde_json::to_writer(&mut writer, &res)?;
                writer.flush()?;
                debug!("Response sent to {}: {:?}", peer_addr, res);
            }
            Request::Remove { key } => {
                let res = match engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok(()),
                    Err(err) => RemoveResponse::Err(format!("{}", err)),
                };

                serde_json::to_writer(&mut writer, &res)?;
                writer.flush()?;
                debug!("Response sent to {}: {:?}", peer_addr, res);
            }
        }
    }

    Ok(())
}
