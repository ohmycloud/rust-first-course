use anyhow::Result;
use course_proto::pb::abi::{CommandResponse, KvPair, Value};

use crate::error::KvError;

pub trait Storage {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, KvError>;
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    fn get_all(&self, table: &str) -> Result<Vec<KvPair>, KvError>;
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>>;
}

pub trait CommandService {
    /// 处理 Command, 返回 Response
    fn execute(self, store: &impl Storage) -> CommandResponse;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::MemTable;

    fn test_basi_interface(store: impl Storage) {
        let v = store.set("t1", "language".into(), "Perl 6".into());
        let v1 = store.set("t1", "language".into(), "Raku".into());
        assert_eq!(v1.unwrap(), Some("Perl 6".into()));

        let v = store.get("t1", "language");
        assert_eq!(v.unwrap(), Some("Raku".into()));

        assert_eq!(None, store.get("t1", "Raku".into()).unwrap());
        assert!(store.get("t2", "language").unwrap().is_none());

        assert_eq!(store.contains("t1", "language").unwrap(), true);
        assert_eq!(store.contains("t1", "lan").unwrap(), false);
        assert_eq!(store.contains("t2", "language").unwrap(), false);

        let v = store.del("t1", "language").unwrap();
        assert_eq!(v, Some("Raku".into()));

        assert_eq!(None, store.del("t1", "Raku").unwrap());
        assert_eq!(None, store.del("t2", "Raku").unwrap());
    }

    fn test_get_all(store: impl Storage) {
        store.set("t2", "k1".into(), "v1".into()).unwrap();
        store.set("t2", "k2".into(), "v2".into()).unwrap();

        let mut data = store.get_all("t2").unwrap();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(
            data,
            vec![
                KvPair::new("k1", "v1".into()),
                KvPair::new("k2", "v2".into()),
            ]
        );
    }

    fn test_get_iter(store: impl Storage) {
        store.set("t2", "k1".into(), "v1".into()).unwrap();
        store.set("t2", "k2".into(), "v2".into()).unwrap();

        let mut data: Vec<_> = store.get_iter("t2").unwrap().collect();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                KvPair::new("k1", "v1".into()),
                KvPair::new("k2", "v2".into()),
            ]
        );
    }

    #[test]
    fn memtable_basic_interface_should_work() {
        let store = MemTable::new();
        test_basi_interface(store);
    }

    #[test]
    fn memtable_get_all_should_work() {
        let store = MemTable::new();
        //test_get_all(store);
    }
}
