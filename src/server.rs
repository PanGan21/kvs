use std::net::SocketAddr;

use futures::{SinkExt, StreamExt, TryFutureExt};
use log::error;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tokio_serde::{formats::SymmetricalJson, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{KvsEngine, Request, Response, Result};

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
    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        while let Ok((tcp, _)) = listener.accept().await {
            let engine = self.engine.clone();
            tokio::spawn(serve(engine, tcp).map_err(|e| error!("Error on serving client: {}", e)));
        }

        Ok(())
    }
}

async fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> Result<()> {
    let (read_half, write_half) = io::split(tcp);

    let mut read_json = SymmetricallyFramed::new(
        FramedRead::new(read_half, LengthDelimitedCodec::new()),
        SymmetricalJson::default(),
    );

    let mut write_json = SymmetricallyFramed::new(
        FramedWrite::new(write_half, LengthDelimitedCodec::new()),
        SymmetricalJson::default(),
    );

    while let Some(req) = read_json.next().await {
        let engine = engine.clone();
        let resp = match req? {
            Request::Get { key } => Response::Get(engine.get(key).await?),
            Request::Set { key, value } => {
                engine.set(key, value).await?;
                Response::Set
            }
            Request::Remove { key } => {
                let res = engine.remove(key).await;
                match res {
                    Ok(_) => Response::Remove,
                    Err(e) => Response::Err(e.to_string()),
                }
            }
        };

        write_json.send(resp).await?;
    }

    Ok(())
}
