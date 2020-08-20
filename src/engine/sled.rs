use crate::Result;
use super::KvsEngine;
use sled::Db;
use std::fs;
use std::path::PathBuf;
use crate::KvStoreError;

pub struct SledKvsEngine {
    bd: Db
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        let bd: Db = sled::open(&path)?;
        Ok(
            SledKvsEngine{bd}
        )
    }
}

impl Clone for SledKvsEngine {

    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl KvsEngine for SledKvsEngine {

    fn set(&self, key: String, value: String) -> Result<()> {
        self.bd.insert(key.into_bytes(), value.into_bytes())?;
        self.bd.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.bd.get(key.into_bytes())?
            .map(|i_vec| AsRef::<[u8]>::as_ref(&i_vec).to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&self, key: String) -> Result<()> {
        self.bd.remove(key.to_owned())?.ok_or(KvStoreError::KeyNotFound)?;
        self.bd.flush()?;
        Ok(())
    }
}
