use std::{
    io::{BufReader, BufWriter},
    net::{SocketAddr, TcpListener, TcpStream},
};

use log::{debug, error, info};
use serde_json::Deserializer;

use crate::{KvsEngine, Request, Result};

/// The server of the key value store.
pub struct KvsServer<T: KvsEngine> {
    engine: T,
}

impl<T: KvsEngine> KvsServer<T> {
    /// Create a `KvsServer` with a given storage engine.
    pub fn new(engine: T) -> Self {
        KvsServer { engine }
    }

    /// Run the server listening on the given address
    pub fn run(&mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        error!("Error on serving client: {}", e);
                    }
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        for req in req_reader {
            debug!("Receive request from {}: {:?}", peer_addr, req);
        }

        Ok(())
    }
}
