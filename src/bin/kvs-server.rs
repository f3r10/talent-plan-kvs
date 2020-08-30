extern crate slog;
use kvs::KvStoreError;
use std::io::prelude::*;

use clap::{App, Arg};
use kvs::KvStore;
use kvs::RayonThreadPool;
use kvs::SharedQueueThreadPool;
use kvs::SledKvsEngine;
use kvs::ThreadPool;
use kvs::{KvsServer, Result};
use slog::{info, o, Drain};
use std::env::current_dir;
use std::fs::OpenOptions;
use std::net::SocketAddr;

fn main() -> Result<()> {
    let log_path = "stderr";
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    info!(log, "CARGO_PKG_VERSION: {}", env!("CARGO_PKG_VERSION"));
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::from_usage(
                "--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT",
            )
            .help("accepts an IP address with port")
            .default_value("127.0.0.1:4000"),
        )
        .arg(
            Arg::from_usage("--engine [IP-PORT] Optionally which engine should be started")
                .help("engine: kvs or sled")
                .default_value("kvs"),
        )
        .get_matches();
    let engine = matches.value_of("engine").unwrap_or("kvs");
    info!(log, "Engine: {}", engine);
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    info!(log, "Addr: {}", addr);
    start_server(engine.to_owned(), addr.to_owned())
}

fn start_server(engine: String, addr: String) -> Result<()> {
    let path = current_dir()?;
    let engine_check = check_engine(engine.to_owned())?;
    if engine_check {
        match engine.as_ref() {
            "kvs" => {
                let store: KvStore = KvStore::open(path)?;
                let thread_pool = RayonThreadPool::new(4).unwrap();
                let server = KvsServer::new(store, thread_pool)?;
                server.run(addr.parse::<SocketAddr>()?)
            }
            "sled" => {
                let store = SledKvsEngine::open(path)?;
                let thread_pool = SharedQueueThreadPool::new(4).unwrap();
                let server = KvsServer::new(store, thread_pool)?;
                server.run(addr.parse::<SocketAddr>()?)
            }
            _ => unreachable!(),
        }
    } else {
        Err(KvStoreError::EngineError)
    }
}
fn check_engine(engine: String) -> Result<bool> {
    let path = current_dir()?;
    let file = path.join("config.log");
    let mut config = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(&file)?;
    let size = config.metadata()?.len();
    if size > 0 {
        let mut engine_info = String::new();
        config.read_to_string(&mut engine_info)?;
        Ok(engine_info.as_ref() == engine)
    } else {
        config.write_all(engine.as_bytes())?;
        Ok(true)
    }
}
