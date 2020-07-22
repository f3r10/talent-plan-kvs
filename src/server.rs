use std::net::{TcpListener, TcpStream};
use super::Result;
use std::io::prelude::*;
use serde_json::Deserializer;
use super::kvs;
use std::env::current_dir;
use kvs::KvStore;
use super::helper;

pub struct KvsServer {
    engine: KvStore
}

impl KvsServer{
    pub fn new(_engine: String) -> Result<Self> {
        let path = current_dir()?;
        let store: KvStore = KvStore::open(path)?;
        Ok(KvsServer{engine: store})
    }

    pub fn run(mut self, addr: String) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let stream = stream?;
             self.handle_connection(stream)?;
            println!("Connection established");
        }
        Ok(())
    }

    pub fn handle_connection(&mut self, stream: TcpStream) -> Result<()> {
        println!("handle_connection");
        let addr = stream.peer_addr()?;
        let mut writer = std::io::BufWriter::new(&stream);
        let reader = std::io::BufReader::new(&stream);
        let request = Deserializer::from_reader(reader).into_iter::<helper::Request>();
        println!("into_iter");
        for req in request {
            let req = req?;
            println!("request: {:?}", req);
            match req {
                helper::Request::Set{key, value} => {
                    match self.engine.set(key, value) {
                        Ok(()) => {
                            serde_json::to_writer(&mut writer, &helper::SetResponse::Ok(()))?;
                            writer.flush()?;
                            println!("response sent to {}", addr);
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::SetResponse::Err(err.to_string()))?;
                            writer.flush()?;
                            println!("response sent to {}", addr);
                        }
                    }
                }
                helper::Request::Rm(key) => {
                    match self.engine.remove(key) {
                        Ok(()) => {
                            serde_json::to_writer(&mut writer, &helper::RmResponse::Ok(()))?;
                            writer.flush()?;
                            println!("response sent to {}", addr);
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::RmResponse::Err(err.to_string()))?;
                            writer.flush()?;
                            println!("response sent to {}", addr);
                        }
                    }
                }
                helper::Request::Get(key) => {
                    match self.engine.get(key) {
                        Ok(value) => {
                            serde_json::to_writer(&mut writer, &helper::GetResponse::Ok(value))?;
                            writer.flush()?;
                            println!("response sent to {}, resp Ok", addr);
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::GetResponse::Err(err.to_string()))?;
                            writer.flush()?;
                            println!("response sent to {}", addr);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
