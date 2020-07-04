use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::fs::{OpenOptions, File};
use std::result;
use std::fmt;
use std::error;
use std::io::{BufWriter, BufReader, SeekFrom};
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
    index: BTreeMap<String, CommandPos>,
    path: PathBuf,
    writer: BufWriterWithPos<File>,
    reader: BufReaderWithPos<File>,
}

#[derive(Debug)]
pub enum KvStoreError{
    SerdeIo(serde_json::Error),
    Io(std::io::Error),
    MapError{details: String},
}
pub type Result<T> = result::Result<T, KvStoreError>;

impl From<std::io::Error> for KvStoreError {
    fn from(err: std::io::Error) -> KvStoreError {
        KvStoreError::Io(err)
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(err: serde_json::Error) -> Self {
        KvStoreError::SerdeIo(err)
    }
}

impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            KvStoreError::SerdeIo(ref err) => write!(f, "{}", err),
            KvStoreError::Io(ref err) => write!(f, "{}", err),
            KvStoreError::MapError{details} => write!(f, "{}", details)
        }
    }
}

impl error::Error for KvStoreError {
    fn description(&self) -> &str {
        match &*self {
            KvStoreError::SerdeIo(ref err) => err.description(),
            KvStoreError::Io(ref err) => err.description(),
            KvStoreError::MapError{details} => &details,
        }
    }
}


#[derive(Debug)]
struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl <R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut reader: R) -> Result<Self> {
        let pos = reader.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos{
            reader: BufReader::new(reader),
            pos
        })
    }
}

impl <R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

impl <R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
   
}
/////////// Writer
#[derive(Debug)]
struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl <W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
   
}

impl <W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut writer: W) -> Result<Self> {
        let pos = writer.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos{
            writer: BufWriter::new(writer),
            pos
        })
    }
   
}

#[derive(Debug, Serialize, Deserialize)]
enum Command{
    Set{key: String, value: String},
    Rm(String),
}

#[derive(Debug)]
struct CommandPos {
    pos: u64,
    len: u64,
}


impl KvStore {
    /// new method would create a new instance of `KvStore`
    // pub fn new() -> KvStore {
    //     KvStore {
    //         map: HashMap::new(),
    //     }
    // }

    // pub fn open<P: AsRef<Path> + Sized>(path: P) -> Result<KvStore> {
    fn deserialize_file(
        reader: &mut BufReaderWithPos<File>,
        index: &mut BTreeMap<String, CommandPos>,
    ) -> Result<()>{
        let mut pos = reader.seek(SeekFrom::Start(0))?;
        let mut stream = serde_json::Deserializer::from_reader(reader).into_iter();
        while let Some(cmd) = stream.next() {
            let current_pos = stream.byte_offset() as u64;
            match cmd? {
                Command::Set{key, ..} => {
                    let pos = CommandPos{pos, len: (current_pos - pos)};
                    index.insert(key, pos)
                }
                Command::Rm(key) => {
                    index.remove(&key)
                }
            };
            pos = current_pos;
        };
        Ok(())
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
        let mut reader = BufReaderWithPos::new(File::open(txt)?)?;
        let mut index: BTreeMap<String, CommandPos> = BTreeMap::new();
        KvStore::deserialize_file(&mut reader, &mut index)?;
        let writer = BufWriterWithPos::new(file)?;
        Ok(KvStore{
            path: path.into(),
            index,
            writer,
            reader,
        })
    }

    /// assing a value to a specific key
    ///
    /// if the key already exists the value is overwritten
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd: Command = Command::Set{key: key.to_owned(), value: value.to_owned()};
        let latest_post = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.writer.flush()?;
        let new_pos = CommandPos{pos: latest_post, len: (self.writer.pos - latest_post)};
        self.index.insert(key, new_pos);
        Ok(())

            // .map_err(KvStoreError::SerdeIo)
            // .and_then(|_| {
            //     self.index.insert(key, self.writer.pos);
            //     Ok(())
            // })
    }

    /// gets the value of a specific key if there is some or none.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(a) => {
                let reader = self.reader.reader.get_mut();
                reader.seek(SeekFrom::Start(a.pos))?;
                let cmd_writer = reader.take(a.len);
                if let Command::Set{value,  ..} = serde_json::from_reader(cmd_writer)? {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// removes the the key and the associated value.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.index.contains_key(&key) {
            true => {
                let cmd: Command = Command::Rm(key.to_owned());
                serde_json::to_writer(&mut self.writer, &cmd)?;
                self.writer.writer.flush()?;
                self.index.remove(&key).expect("Key not found");
                Ok(())
            }
            false => Err(KvStoreError::MapError{details:"Key not found".to_owned()})
        }
    }
}
