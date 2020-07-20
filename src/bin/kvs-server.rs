extern crate slog;
extern crate slog_term;
extern crate slog_async;

use clap::{App, Arg};
use slog::{Drain, o, info};
use std::fs::OpenOptions;
use kvs::{KvsServer, Result};

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
            Arg::from_usage("--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT")
                .help("accepts an IP address with port")
                .default_value("127.0.0.1:4000")
        )
        .arg(
            Arg::from_usage("--engine [IP-PORT] Optionally which engine should be started")
                .help("engine: kvs or sled")
                .default_value("kvs")
        )
        .get_matches();
    let engine = matches.value_of("engine").unwrap_or("kvs");
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");
    let server = KvsServer::new(engine.to_owned());
    server.run(addr.to_owned())
}
