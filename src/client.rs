use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};

use crate::{GetResponse, Request, Result};

/// Key value store client
pub struct KvsClient {
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let tcp_reader = TcpStream::connect(addr)?;
        let tcp_writer = tcp_reader.try_clone()?;
        Ok(KvsClient {
            reader: Deserializer::from_reader(BufReader::new(tcp_reader)),
            writer: BufWriter::new(tcp_writer),
        })
    }

    /// Get the value of a given key from the server.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get { key })?;
        self.writer.flush()?;
        let response = GetResponse::deserialize(&mut self.reader)?;

        match response {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(err) => Err(crate::KvsError::StringError(err)),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        Ok(())
    }
}
