use std::net::SocketAddr;

use tokio::{
    io::{self, ReadHalf, WriteHalf},
    net::TcpStream,
};

use tokio_serde::{
    formats::{Json, SymmetricalJson},
    SymmetricallyFramed,
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{KvsError, Request, Response, Result};
use futures::{SinkExt, StreamExt};

/// Key value store client
pub struct KvsClient {
    read_json: SymmetricallyFramed<
        FramedRead<ReadHalf<TcpStream>, LengthDelimitedCodec>,
        Response,
        Json<Response, Response>,
    >,
    write_json: SymmetricallyFramed<
        FramedWrite<WriteHalf<TcpStream>, LengthDelimitedCodec>,
        Request,
        Json<Request, Request>,
    >,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let tcp = TcpStream::connect(addr).await?;

        let (read_half, write_half) = io::split(tcp);

        let write_json = SymmetricallyFramed::new(
            FramedWrite::new(write_half, LengthDelimitedCodec::new()),
            SymmetricalJson::default(),
        );
        let read_json = SymmetricallyFramed::new(
            FramedRead::new(read_half, LengthDelimitedCodec::new()),
            SymmetricalJson::default(),
        );

        Ok(KvsClient {
            read_json,
            write_json,
        })
    }

    /// Get the value of a given key from the server.
    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        let res = self.send_request(Request::Get { key }).await?;
        match res {
            Response::Get(value) => Ok(value),
            Response::Set | Response::Remove => {
                Err(KvsError::StringError("Invalid response".to_string()))
            }
            Response::Err(e) => Err(KvsError::StringError(e)),
        }
    }

    /// Set the value of a string key in the server.
    pub async fn set(&mut self, key: String, value: String) -> Result<()> {
        let res = self.send_request(Request::Set { key, value }).await?;
        match res {
            Response::Set => Ok(()),
            Response::Remove | Response::Get(_) => {
                Err(KvsError::StringError("Invalid response".to_string()))
            }
            Response::Err(e) => Err(KvsError::StringError(e)),
        }
    }

    /// Remove a string key in the server.
    pub async fn remove(&mut self, key: String) -> Result<()> {
        let res = self.send_request(Request::Remove { key }).await?;
        match res {
            Response::Remove => Ok(()),
            Response::Get(_) | Response::Set => {
                Err(KvsError::StringError("Invalid response".to_string()))
            }
            Response::Err(e) => Err(KvsError::StringError(e)),
        }
    }

    async fn send_request(&mut self, req: Request) -> Result<Response> {
        self.write_json.send(req).await?;
        let response = self
            .read_json
            .next()
            .await
            .ok_or_else(|| crate::KvsError::StringError("No response received".into()))?;

        Ok(response?)
    }
}
