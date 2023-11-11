use sled::Db;

use crate::{KvsEngine, KvsError, Result};

/// Wrapper of `sled::Db
#[derive(Clone)]
pub struct SledKvsEngine(Db);

/// Implementation of SledKvsEngine
impl SledKvsEngine {
    /// Creates a `SledKvsEngine` from `sled::Db`.
    pub fn new(db: Db) -> Self {
        SledKvsEngine(db)
    }
}

/// Implementation of KvsEngine for SledKvsEngine trait
impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        let tree = &self.0;
        tree.insert(key, value.into_bytes())?;
        tree.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let tree = &self.0;
        Ok(tree
            .get(key)?
            .map(|v| v.to_vec())
            .map(String::from_utf8)
            .transpose()?)
    }

    fn remove(&self, key: String) -> Result<()> {
        let tree = &self.0;
        tree.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        tree.flush()?;
        Ok(())
    }
}
