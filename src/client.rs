use std::net::TcpStream;
use super::Result;

pub struct KvsClient{
    reader: TcpStream,
    writer: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: String) -> Result<Self> {
        let reader = TcpStream::connect(addr).unwrap();
        let writer = reader.try_clone().unwrap();
        Ok(KvsClient{reader, writer})
    }

}
