use std::result;
use std::fmt;
use std::error;

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
