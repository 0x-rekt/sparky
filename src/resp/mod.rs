pub mod parser;
pub mod serializer;

#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    BulkString(String),
    Integer(i64),
    Array(Vec<RespValue>),
    Nil,
}