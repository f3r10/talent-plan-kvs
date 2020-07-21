use std::net::{TcpListener, TcpStream};
use super::Result;
use std::io::prelude::*;
use std::str;

pub struct KvsServer {
    engine: String
}

impl KvsServer{
    pub fn new(engine: String) -> Self {
        KvsServer{engine}
    }

    pub fn run(self, addr: String) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let stream = stream?;
            handle_connection(stream)?;
            println!("Connection established");
        }
        Ok(())
    }


}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;
    let request = str::from_utf8(&buffer).unwrap();
    println!("request: {:?}", request);
    stream.write("+PONG\r\n".as_bytes())?;
    stream.flush()?;
    Ok(())
}
