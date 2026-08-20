mod expire;
mod generic;
mod hash;
mod list;
mod server;
mod set;
mod string;

use bytes::Bytes;

use crate::{db::SharedDb, resp::RespValue};

pub fn handle_command(request: RespValue, db: SharedDb) -> RespValue {
    let RespValue::Array(parts) = request else {
        return RespValue::Error("ERR invalid command format".to_string());
    };
    let Some(RespValue::BulkString(command)) = parts.first() else {
        return RespValue::Error("ERR invalid command format".to_string());
    };

    let args = &parts[1..];
    let command = String::from_utf8_lossy(command).to_ascii_uppercase();

    match command.as_str() {
        "PING" | "ECHO" => server::handle(&command, args),
        "SET" | "GET" | "STRLEN" | "MGET" | "MSET" | "INCR" | "DECR" | "INCRBY" | "APPEND"
        | "GETSET" | "GETDEL" => string::handle(&command, args, db),
        "EXPIRE" | "PEXPIRE" | "TTL" | "PTTL" => expire::handle(&command, args, db),
        "DEL" | "EXISTS" | "TYPE" | "RENAME" | "KEYS" => generic::handle(&command, args, db),
        "RPUSH" | "LPUSH" | "LRANGE" | "LPOP" | "RPOP" | "LLEN" | "LINDEX" | "LSET" | "LREM" => {
            list::handle(&command, args, db)
        }
        "SADD" | "SINTER" | "SREM" | "SMEMBERS" | "SISMEMBER" | "SCARD" | "SUNION" | "SDIFF" => {
            set::handle(&command, args, db)
        }
        "HSET" | "HGET" | "HDEL" | "HEXISTS" | "HGETALL" | "HKEYS" | "HVALS" | "HLEN"
        | "HINCRBY" => hash::handle(&command, args, db),
        _ => RespValue::Error(format!("ERR unknown command: {command}")),
    }
}

pub(crate) fn get_string_arg(
    args: &[RespValue],
    index: usize,
    command: &str,
) -> Result<Bytes, RespValue> {
    match args.get(index) {
        Some(RespValue::BulkString(value)) => Ok(value.clone()),
        _ => Err(RespValue::Error(format!(
            "ERR wrong number of arguments for '{}' command",
            command.to_lowercase()
        ))),
    }
}

pub(crate) fn get_nonnegative_integer(args: &[RespValue], index: usize) -> Result<u64, RespValue> {
    let value = get_string_arg(args, index, "expire")
        .map_err(|_| RespValue::Error("ERR invalid expire time".to_string()))?;
    String::from_utf8_lossy(&value)
        .parse::<u64>()
        .map_err(|_| RespValue::Error("ERR invalid expire time".to_string()))
}
