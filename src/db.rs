use rocksdb::{Options, DB, ColumnFamilyDescriptor, BoundColumnFamily, WriteBatch};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use bincode;

/// Simple chain state storage
pub struct ChainStore {
    db: Arc<DB>,
}

impl ChainStore {
    /// Open or create the RocksDB database at the given path
    pub fn open(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new("blocks", Options::default()),
            ColumnFamilyDescriptor::new("txs", Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cfs).expect("failed to open RocksDB");
        Self { db: Arc::new(db) }
    }

    /// Store a serializable object under a key in a column family
    pub fn put<T: Serialize>(&self, cf_name: &str, key: &[u8], value: &T) -> anyhow::Result<()> {
        let cf_handle = self.db.cf_handle(cf_name).expect("missing CF");
        let bytes = bincode::serialize(value)?;
        self.db.put_cf(&cf_handle, key, bytes)?;
        Ok(())
    }

    /// Retrieve a deserializable object by key from a column family
    pub fn get<T: DeserializeOwned>(&self, cf_name: &str, key: &[u8]) -> anyhow::Result<Option<T>> {
        let cf_handle = self.db.cf_handle(cf_name).expect("missing CF");
        match self.db.get_cf(&cf_handle, key)? {
            Some(bytes) => {
                let val: T = bincode::deserialize(&bytes)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    /// Batch insert multiple key/value pairs in one atomic operation
    pub fn batch_put<T: Serialize>(
        &self,
        cf_name: &str,
        entries: Vec<(&[u8], &T)>,
    ) -> anyhow::Result<()> {
        let cf_handle = self.db.cf_handle(cf_name).expect("missing CF");
        let mut batch = WriteBatch::default();
        for (key, value) in entries {
            let bytes = bincode::serialize(value)?;
            batch.put_cf(&cf_handle, key, bytes);
        }
        self.db.write(batch)?;
        Ok(())
    }
}
