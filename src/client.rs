use std::net::TcpStream;
use super::Result;
use std::io::prelude::*;
use std::str;

pub struct KvsClient{
    reader: TcpStream,
    writer: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: String) -> Result<Self> {
        let reader = TcpStream::connect(addr)?;
        let writer = reader.try_clone()?;
        Ok(KvsClient{reader, writer})
    }

    pub fn get(mut self, key: String) -> Result<()> {
        self.writer.write(b"algo")?;
        let mut buffer = [0; 512];
        self.reader.read(&mut buffer[..])?;
        let answer_msg = str::from_utf8(&buffer).unwrap();
        println!("answer: {:?}", answer_msg);
        Ok(())
    }
}
