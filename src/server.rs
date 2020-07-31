use std::net::{TcpListener, TcpStream};
use super::Result;
use std::io::prelude::*;
use serde_json::Deserializer;
use crate::engine::KvsEngine;
use super::helper;
use std::net::SocketAddr;

pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E>{
    pub fn new(engine: E) -> Result<Self> {
        Ok(KvsServer{engine})
    }

    pub fn run(mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let stream = stream?;
             self.handle_connection(stream)?;
            println!("Connection established");
        }
        Ok(())
    }

    pub fn handle_connection(&mut self, stream: TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        println!("server listen on {}", addr);
        let mut writer = std::io::BufWriter::new(&stream);
        let reader = std::io::BufReader::new(&stream);
        let request = Deserializer::from_reader(reader).into_iter::<helper::Request>();
        for req in request {
            let req = req?;
            match req {
                helper::Request::Set{key, value} => {
                    match self.engine.set(key, value) {
                        Ok(()) => {
                            serde_json::to_writer(&mut writer, &helper::SetResponse::Ok(()))?;
                            writer.flush()?;
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::SetResponse::Err(err.to_string()))?;
                            writer.flush()?;
                        }
                    }
                }
                helper::Request::Rm(key) => {
                    match self.engine.remove(key) {
                        Ok(()) => {
                            serde_json::to_writer(&mut writer, &helper::RmResponse::Ok(()))?;
                            writer.flush()?;
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::RmResponse::Err(err.to_string()))?;
                            writer.flush()?;
                        }
                    }
                }
                helper::Request::Get(key) => {
                    match self.engine.get(key) {
                        Ok(value) => {
                            serde_json::to_writer(&mut writer, &helper::GetResponse::Ok(value))?;
                            writer.flush()?;
                        },
                        Err(err) => {
                            serde_json::to_writer(&mut writer, &helper::GetResponse::Err(err.to_string()))?;
                            writer.flush()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
