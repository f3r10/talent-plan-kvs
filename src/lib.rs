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

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

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
    current_log: u64,
    uncompacted: u64,
}

#[derive(Debug)]
pub enum KvStoreError{
    SerdeIo(serde_json::Error),
    Io(std::io::Error),
    KeyNotFound,
}
pub type Result<T> = result::Result<T, KvStoreError>;


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
        remove_empty_logs(&path)?;
        let log_ids = get_log_ids(&path)?;
        let mut index: BTreeMap<String, CommandPos> = BTreeMap::new();
        let mut readers: HashMap<u64, BufReaderWithPos<File>> = HashMap::new();
        let mut uncompacted = 0;
        for &id in &log_ids {
            let mut reader = BufReaderWithPos::new(File::open(construct_file(id, &path))?)?;
            uncompacted += deserialize_cmds(&mut reader, &mut index, id)?;
            readers.insert(id, reader);
        }
        let last_log_to_write = log_ids.last().unwrap_or(&0) + 1;
        let writer = create_new_writer_log(&path, last_log_to_write, &mut readers)?;
        Ok(KvStore{
            path,
            index,
            writer,
            readers,
            current_log: last_log_to_write,
            uncompacted,
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
        if let Some(cmd_old) = self.index.insert(key, new_pos) {
            self.uncompacted += cmd_old.len
        }
        if self.uncompacted > COMPACTION_THRESHOLD {
            // Should this operation done in a different thread.
            self.compaction()?
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    // This operation needs to create a new log compacted file in which only the latest cmd are going to be added.
    // In order to achieve this is necessary to:
    // 1. Create an log file N in which all the previous commands are going to be compacted. This means increments the current id.
    // 2. The current log is updated with the next reference after the compacted file.
    // 3. All the files previous to N are going to be deleted.
    // 4. A future open call  would read only from the new created and compated file.
    // 5. Possible new operations would use a file which id is the next after the compacted file.
    ///////////////////////////////////////////////////////////////////////////

    fn compaction(&mut self) -> Result<()> {
        let compacted_log_file_id = self.current_log + 1;
        self.current_log += 2;
        self.writer = create_new_writer_log(&self.path, self.current_log , &mut self.readers)?;
        let mut compacted_writer = create_new_writer_log(&self.path, compacted_log_file_id, &mut self.readers)?;
        let mut new_pos = 0;
        for cmd_log in &mut self.index.values_mut() {
            let reader = self.readers
                             .get_mut(&cmd_log.log_id)
                             .expect("Unable to find log");
            if reader.pos != cmd_log.pos {
                reader.seek(SeekFrom::Start(cmd_log.pos))?;
            }
            let mut cmd_writer = reader.take(cmd_log.len);
            let len = std::io::copy(&mut cmd_writer, &mut compacted_writer)?;
            let new_cmd_log = CommandPos{
                pos: (new_pos + len) - new_pos,
                len,
                log_id: compacted_log_file_id,
            };
            *cmd_log = new_cmd_log;
            new_pos += len;
        }
        compacted_writer.flush()?;

        let stale_files: Vec<_> = self.readers
                             .keys()
                             .filter(|&&key| key < compacted_log_file_id)
                             .cloned()
                             .collect();
        for file_id in stale_files {
            self.readers.remove(&file_id);
            fs::remove_file(construct_file(file_id, &self.path))?;
        }
        self.uncompacted = 0;
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
        if let Some(cmd_old) = self.index.remove(&key) {
            let cmd: Command = Command::Rm(key.to_owned());
            serde_json::to_writer(&mut self.writer, &cmd).unwrap();
            self.uncompacted += cmd_old.len;
            self.writer.writer.flush().unwrap();
            Ok(())
        } else {
            Err(KvStoreError::KeyNotFound)
        }
    }
}

fn deserialize_cmds(
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
    log_id: u64,
) -> Result<u64>{
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter();
    let mut uncompacted = 0;
    while let Some(cmd) = stream.next() {
        let current_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set{key, ..} => {
                let pos = CommandPos{pos, len: (current_pos - pos), log_id};
                if let Some(old_cmd) = index.insert(key, pos) {
                    uncompacted += old_cmd.len
                }
            }
            Command::Rm(key) => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len
                }
                // in the next compaction process this remove cmd  entry has to be deleted too because
                // it would not be necessary anymore.
                uncompacted += current_pos - pos
            }
        };
        pos = current_pos;
    };
    Ok(uncompacted)
}

fn remove_empty_logs(path: & PathBuf) -> Result<()> {
    let files = fs::read_dir(path)?;
    let target = std::ffi::OsString::from("log");
    files
        .filter_map(std::io::Result::ok)
        .map(|e| e.path())
        .filter(|entry| entry.is_file() && entry.extension() == Some(&target))
        .flat_map(|e| {
            let met = fs::metadata(&e);
            if met.unwrap().len() == 0 {
                fs::remove_file(e)
            } else {
                Ok(())
            }
        })
        .for_each(|_| ());
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


fn create_new_writer_log(
    path: &PathBuf,
    id: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let log = construct_file(id, &path);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
        .append(true)
        .create(true)
        .write(true)
        .open(&log)?
    )?;
    let reader = BufReaderWithPos::new(File::open(log)?)?;
    readers.insert(id, reader);
    Ok(writer)

}
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
