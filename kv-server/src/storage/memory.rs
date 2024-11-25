use anyhow::Result;
use course_proto::pb::abi::{KvPair, Value};
use dashmap::{mapref::one::Ref, DashMap};

use crate::{command::Storage, error::KvError};

#[derive(Clone, Debug, Default)]
pub struct MemTable {
    pub tables: DashMap<String, DashMap<String, Value>>,
}

impl MemTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// 如果名为 name 的 hash table 不存在, 则创建, 否则返回
    fn get_or_create_table(&self, name: &str) -> Ref<String, DashMap<String, Value>> {
        match self.tables.get(name) {
            Some(table) => table,
            None => {
                let entry = self.tables.entry(name.into()).or_default();
                entry.downgrade()
            }
        }
    }
}

impl Storage for MemTable {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        let value = table.get(key).map(|v| v.value().clone());
        Ok(value)
    }

    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        let value = table.insert(key, value);
        Ok(value)
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError> {
        let table = self.get_or_create_table(table);
        let value = table.contains_key(key);
        Ok(value)
    }

    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        let value = table.remove(key).map(|(_k, v)| v);
        Ok(value)
    }

    fn get_all(&self, table: &str) -> Result<Vec<KvPair>, KvError> {
        let table = self.get_or_create_table(table);
        let value = table
            .iter()
            .map(|v| KvPair::new(v.key(), v.value().clone()))
            .collect();
        Ok(value)
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>> {
        todo!()
    }
}
