extern crate clap;
use clap::{App, Arg, SubCommand};
use kvs::{Result, KvsClient};
use std::net::SocketAddr;
use kvs::KvStoreError;
use std::process::exit;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("get")
                .about("get a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::from_usage("--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT")
                        .help("accepts an IP address with port")
                        .default_value("127.0.0.1:4000")
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(Arg::with_name("VALUE").help("value").required(true))
                .arg(
                    Arg::from_usage("--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT")
                        .help("accepts an IP address with port")
                        .default_value("127.0.0.1:4000")
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("delete a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::from_usage("--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT")
                        .help("accepts an IP address with port")
                        .default_value("127.0.0.1:4000")
                ),
        )
        .get_matches();


    match matches.subcommand() {
        ("get", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let addr = _matches.value_of("addr").unwrap_or("127.0.0.1:4000");
            let mut client = KvsClient::connect(addr.parse::<SocketAddr>()?)?;
            let value_o = client.get(key.to_owned())?;
            match value_o {
                Some(v) => println!("{}", v),
                None => println!("Key not found"),
            }
        }
        ("set", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let value = _matches.value_of("VALUE").expect("VALUE argument missing");
            let addr = _matches.value_of("addr").unwrap_or("127.0.0.1:4000");
            let mut client = KvsClient::connect(addr.parse::<SocketAddr>()?)?;
            client.set(key.to_owned(), value.to_owned())?;
        }
        ("rm", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let addr = _matches.value_of("addr").unwrap_or("127.0.0.1:4000");
            let mut client = KvsClient::connect(addr.parse::<SocketAddr>()?)?;
            match client.rm(key.to_owned()) {
                Ok(()) => {}
                Err(KvStoreError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);

                },
                Err(e) => {
                    println!("error: {:?}", e);
                    return Err(e)
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
