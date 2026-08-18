use bytes::Bytes;

pub mod parser;
pub mod serializer;

#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    BulkString(Bytes),
    Integer(i64),
    Array(Vec<RespValue>),
    Nil,
}
