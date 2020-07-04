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
use std::fs;
use std::ffi::OsStr;

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
    readers: HashMap<u64, BufReaderWithPos<File>>,
    current_log: u64
}

#[derive(Debug)]
pub enum KvStoreError{
    SerdeIo(serde_json::Error),
    Io(std::io::Error),
    KeyNotFound,
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
            KvStoreError::SerdeIo(ref err) => err.fmt(f),
            KvStoreError::Io(ref err) => err.fmt(f),
            KvStoreError::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

impl error::Error for KvStoreError {
    fn description(&self) -> &str {
        match *self {
            KvStoreError::SerdeIo(ref err) => err.description(),
            KvStoreError::Io(ref err) => err.description(),
            KvStoreError::KeyNotFound => "Key not found",
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
    log_id: u64,
}


fn deserialize_file(
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
    log_id: u64,
) -> Result<()>{
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter();
    while let Some(cmd) = stream.next() {
        let current_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set{key, ..} => {
                let pos = CommandPos{pos, len: (current_pos - pos), log_id};
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

fn get_log_ids(path: &PathBuf) -> Result<Vec<u64>> {
    let files = fs::read_dir(path)?;
    let target = std::ffi::OsString::from("log");
    let mut a: Vec<u64> = files
        .filter_map(std::io::Result::ok)
        .map(|e| e.path())
        .filter(|entry| entry.is_file() && entry.extension() == Some(&target))
        .flat_map(|entry| {
            entry.file_name()
                 .and_then(OsStr::to_str)
                 .map(|file| file.trim_end_matches(".log"))
                 .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    a.sort_unstable();
    Ok(a)

}

fn construct_file(id: u64, path: &PathBuf) -> PathBuf {
    path.join(format!("{}.log", id))
}
impl KvStore {
    /// new method would create a new instance of `KvStore`
    // pub fn new() -> KvStore {
    //     KvStore {
    //         map: HashMap::new(),
    //     }
    // }

    // pub fn open<P: AsRef<Path> + Sized>(path: P) -> Result<KvStore> {

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        let log_ids = get_log_ids(&path)?;
        let mut index: BTreeMap<String, CommandPos> = BTreeMap::new();
        let mut readers: HashMap<u64, BufReaderWithPos<File>> = HashMap::new();
        for &id in &log_ids {
            let mut reader = BufReaderWithPos::new(File::open(construct_file(id, &path))?)?;
            deserialize_file(&mut reader, &mut index, id)?;
            readers.insert(id, reader);
        }
        let last_log_to_write = log_ids.last().unwrap_or(&0) + 1;
        let log = construct_file(last_log_to_write, &path);
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&log)
            .expect("Cannot open file");
        let writer = BufWriterWithPos::new(file)?;
        let reader = BufReaderWithPos::new(File::open(log)?)?;
        readers.insert(last_log_to_write, reader);
        Ok(KvStore{
            path,
            index,
            writer,
            readers,
            current_log: last_log_to_write,
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
        let new_pos = CommandPos{pos: latest_post, len: (self.writer.pos - latest_post), log_id: self.current_log};
        self.index.insert(key, new_pos);
        Ok(())
    }

    /// gets the value of a specific key if there is some or none.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(a) => {
                let reader = self.readers
                                 .get_mut(&a.log_id)
                                 .expect("unable to find current log");
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
            false => Err(KvStoreError::KeyNotFound)
        }
    }
}
