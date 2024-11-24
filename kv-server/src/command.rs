use course_proto::pb::abi::{CommandResponse, KvPair, Value};

/*
pub trait Storage {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value, KvError>>;
    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value, KvError>>;
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    fn get_all(&self, table: &str) -> Result<Vec<KvPair>, KvError>;
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = KvPair>>>;
}

pub trait CommandService {
    /// 处理 Command, 返回 Response
    fn execute(self, store: &impl Storage) -> CommandResponse;
}
*/
