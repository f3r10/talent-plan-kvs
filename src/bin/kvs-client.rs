extern crate clap;
use clap::{App, Arg, SubCommand};
use std::process;
use kvs::{KvStore, Result, KvsClient};
use std::env::current_dir;

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::from_usage("--addr [IP-PORT] Optionally accepts an IP address, with the format IP:PORT")
                .help("accepts an IP address with port")
                .default_value("127.0.0.1:4000")
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(Arg::with_name("VALUE").help("value").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("delete a value")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:4000");

    match matches.subcommand() {
        ("get", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let client = KvsClient::connect(addr.to_owned()).unwrap();
            // let path = current_dir()?;
            // let mut store = KvStore::open(path)?;
            // let value_o = store.get(key.to_owned())?;
            // match value_o {
            //     Some(v) => Ok(println!("{}", v)),
            //     None => Ok(println!("Key not found")),
            // }
        }
        ("set", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let value = _matches.value_of("VALUE").expect("VALUE argument missing");
            let client = KvsClient::connect(addr.to_owned()).unwrap();
            // let path = current_dir()?;
            // let mut store = KvStore::open(path)?;
            // store.set(key.to_owned(), value.to_owned())
        }
        ("rm", Some(_matches)) => {
            let key = _matches.value_of("KEY").expect("KEY argument missing");
            let client = KvsClient::connect(addr.to_owned()).unwrap();
            // let path = current_dir()?;
            // let mut store = KvStore::open(path)?;
            // store.remove(key.to_owned())
        }
        _ => unreachable!(),
    };
    Ok(())
}
