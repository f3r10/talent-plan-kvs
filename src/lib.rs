use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{OpenOptions, File};
use std::fs;
use std::result;
use std::fmt;
use std::io::Cursor;
use std::io::{BufWriter, BufReader};
use serde::{Serialize, Deserialize};
use serde_json;
use std::io::prelude::*;

/// This is an example doc test
///
/// Key/value are stores in-memory and not is disk
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()))
/// ```
#[derive(Debug)]
pub struct KvStore {
    map: HashMap<String, String>,
    path: PathBuf,
    writer: File,
}

#[derive(Debug)]
pub enum KvStoreError{
    SerdeIo(serde_json::Error),
    Io(std::io::Error),
    MapError(String)
}
pub type Result<T> = result::Result<T, KvStoreError>;

impl From<std::io::Error> for KvStoreError {
    fn from(err: std::io::Error) -> KvStoreError {
        KvStoreError::Io(err)
    }
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KvStore Error")
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Command{
    Get(String),
    Set{key: String, value: String},
    Rm(String),
}


impl KvStore {
    /// new method would create a new instance of `KvStore`
    // pub fn new() -> KvStore {
    //     KvStore {
    //         map: HashMap::new(),
    //     }
    // }

    // pub fn open<P: AsRef<Path> + Sized>(path: P) -> Result<KvStore> {
    fn deserialize_file(file: &File) -> HashMap<String, String>{
        let mut map: HashMap<String, String> = HashMap::new();
        let buf_reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(buf_reader);
        while let Ok(u) = Command::deserialize(&mut de) {
            match u {
                Command::Set{key, value} => map.insert(key, value),
                Command::Rm(key) => map.remove(&key),
                Command::Get(_) => Option::None,
            };
        };
        return map;
    }
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let txt = path.join("cmd.json");
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&txt)
            .expect("Cannot open file");
        let in_memory_ref = KvStore::deserialize_file(&file);
        Ok(KvStore{
            path: path.into(),
            map: in_memory_ref,
            writer: file,
        })
    }

    /// assing a value to a specific key
    ///
    /// if the key already exists the value is overwritten
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd: Command = Command::Set{key: key.to_owned(), value: value.to_owned()};
        serde_json::to_writer(&mut self.writer, &cmd)
            .map_err(KvStoreError::SerdeIo)
            .and_then(|_| {
                self.map.insert(key, value);
                Ok(())
            })
    }

    /// gets the value of a specific key if there is some or none.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let cmd: Command = Command::Get(key.to_owned());
        match self.map.get(&key) {
            Some(a) => Ok(Some(a.to_owned())),
            None => Ok(None),
        }
    }

    /// removes the the key and the associated value.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.map.contains_key(&key) {
            true => {
                let cmd: Command = Command::Rm(key.to_owned());
                serde_json::to_writer(&mut self.writer, &cmd)
                    .map_err(KvStoreError::SerdeIo)
                    .and_then(|_| {
                        self.map.remove(&key)
                                .ok_or(KvStoreError::MapError("Unable to remove item".to_owned()))
                                .and_then(|_| Ok(()))
                    })
            }
            false => Ok(())
        }
    }
}
