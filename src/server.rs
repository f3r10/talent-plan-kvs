use super::helper;
use super::Result;
use crate::engine::KvsEngine;
use crate::SharedQueueThreadPool;
use crate::ThreadPool;
use serde_json::Deserializer;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};

pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(engine: E) -> Result<Self> {
        Ok(KvsServer { engine })
    }

    pub fn run(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        let pool = SharedQueueThreadPool::new(4)?;
        for stream in listener.incoming() {
            let stream = stream?;
            let engine = self.engine.clone();
            pool.spawn(move || {
                // panic!("oh no!");
                handle_connection(engine, stream).expect("Thread operations");
                println!("Connection established");
            })
        }
        Ok(())
    }
}

fn handle_connection<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    let addr = stream.peer_addr()?;
    println!("server listen on {}", addr);
    let mut writer = std::io::BufWriter::new(&stream);
    let reader = std::io::BufReader::new(&stream);
    let request = Deserializer::from_reader(reader).into_iter::<helper::Request>();
    for req in request {
        let req = req?;
        match req {
            helper::Request::Set { key, value } => match engine.set(key, value) {
                Ok(()) => {
                    serde_json::to_writer(&mut writer, &helper::SetResponse::Ok(()))?;
                    writer.flush()?;
                }
                Err(err) => {
                    serde_json::to_writer(&mut writer, &helper::SetResponse::Err(err.to_string()))?;
                    writer.flush()?;
                }
            },
            helper::Request::Rm(key) => match engine.remove(key) {
                Ok(()) => {
                    serde_json::to_writer(&mut writer, &helper::RmResponse::Ok(()))?;
                    writer.flush()?;
                }
                Err(err) => {
                    serde_json::to_writer(&mut writer, &helper::RmResponse::Err(err.to_string()))?;
                    writer.flush()?;
                }
            },
            helper::Request::Get(key) => match engine.get(key) {
                Ok(value) => {
                    serde_json::to_writer(&mut writer, &helper::GetResponse::Ok(value))?;
                    writer.flush()?;
                }
                Err(err) => {
                    serde_json::to_writer(&mut writer, &helper::GetResponse::Err(err.to_string()))?;
                    writer.flush()?;
                }
            },
        }
    }
    Ok(())
}
