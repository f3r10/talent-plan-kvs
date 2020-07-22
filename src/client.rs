use std::net::TcpStream;
use super::Result;
use serde_json::de::{Deserializer, IoRead};
use serde::Deserialize;
use super::helper;
use std::io::{BufWriter, BufReader, Write};

pub struct KvsClient{
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
    writer: BufWriter<TcpStream>,
    reader2: BufReader<TcpStream>,
}


impl KvsClient {
    pub fn connect(addr: String) -> Result<Self> {
        let reader = TcpStream::connect(addr)?;
        let reader2 = reader.try_clone()?;
        let writer = reader.try_clone()?;
        Ok(KvsClient{
            reader: Deserializer::from_reader(BufReader::new(reader)),
            writer: BufWriter::new(writer),
            reader2: BufReader::new(reader2),
        })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let mut cmd = helper::Request::Get(key);
        println!("cmd");
        serde_json::to_writer(&mut self.writer, &mut cmd)?;
        self.writer.flush()?;
        println!("write");
        let mut result2 = Deserializer::from_reader(&mut self.reader2); // this can be abstracted on the struct definition
        let result3 = helper::GetResponse::deserialize(&mut result2)?;
        println!("read");
        match result3 {
            helper::GetResponse::Ok(Some(value)) => Ok(Some(value)),
            helper::GetResponse::Ok(None) => Ok(None),
            helper::GetResponse::Err(err) => Err(crate::KvStoreError::ServerResponseErr(err))
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut cmd = helper::Request::Set{key, value};
        serde_json::ser::to_writer(&mut self.writer, &mut cmd)?;
        self.writer.flush()?;
        let result = helper::SetResponse::deserialize(&mut self.reader)?;
        match result {
            helper::SetResponse::Ok(_) => Ok(()),
            helper::SetResponse::Err(err) => Err(crate::KvStoreError::ServerResponseErr(err))
        }
    }

    pub fn rm(&mut self, key: String) -> Result<()> {
        let mut cmd = helper::Request::Rm(key);
        serde_json::ser::to_writer(&mut self.writer, &mut cmd)?;
        self.writer.flush()?;
        let result:helper::RmResponse = helper::RmResponse::deserialize(&mut self.reader)?;
        match result {
            helper::RmResponse::Ok(()) => Ok(()),
            helper::RmResponse::Err(err) => Err(crate::KvStoreError::ServerResponseErr(err))
        }
    }
}
