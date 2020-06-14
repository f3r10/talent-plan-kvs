extern crate clap;
use clap::{App, Arg, SubCommand};
use std::process;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
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

    match matches.subcommand() {
        ("get", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1)
        }
        ("set", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1)
        }
        ("rm", Some(_matches)) => {
            eprintln!("unimplemented");
            process::exit(1)
        }
        _ => unreachable!(),
    }
}
