use std::result;
use std::fmt;
use sled;

pub enum KvStoreError{
    SerdeIo(serde_json::Error),
    Io(std::io::Error),
    KeyNotFound,
    ServerResponseErr(String),
    SledError(sled::Error),
    StringUtf8Error(std::string::FromUtf8Error),
    EngineError,
    AddrParseError(std::net::AddrParseError),
}
pub type Result<T> = result::Result<T, KvStoreError>;


impl From<std::net::AddrParseError> for KvStoreError {
    fn from(err: std::net::AddrParseError) -> KvStoreError {
        KvStoreError::AddrParseError(err)
    }
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

impl From<sled::Error> for KvStoreError {
    fn from(err: sled::Error) -> Self {
        KvStoreError::SledError(err)
    }

}

impl From<std::string::FromUtf8Error> for KvStoreError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        KvStoreError::StringUtf8Error(err)
    }
}


impl fmt::Debug for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            KvStoreError::SerdeIo(ref err) => err.fmt(f),
            KvStoreError::Io(ref err) => err.fmt(f),
            KvStoreError::KeyNotFound => write!(f, "Key not found"),
            KvStoreError::ServerResponseErr(ref err) => err.fmt(f),
            KvStoreError::SledError(ref err) => err.fmt(f),
            KvStoreError::StringUtf8Error(ref err) => err.fmt(f),
            KvStoreError::EngineError => write!(f, "Already created an engine conf"),
            KvStoreError::AddrParseError(ref err) => err.fmt(f),
        }
    }
   
}
impl fmt::Display for KvStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            KvStoreError::SerdeIo(ref err) => err.fmt(f),
            KvStoreError::Io(ref err) => err.fmt(f),
            KvStoreError::KeyNotFound => write!(f, "Key not found"),
            KvStoreError::ServerResponseErr(ref err) => err.fmt(f),
            KvStoreError::SledError(ref err) => err.fmt(f),
            KvStoreError::StringUtf8Error(ref err) => err.fmt(f),
            KvStoreError::EngineError => write!(f, "Already created an engine conf"),
            KvStoreError::AddrParseError(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for KvStoreError {}
