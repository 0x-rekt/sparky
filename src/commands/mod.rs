mod expire;
mod generic;
mod hash;
mod list;
mod server;
mod set;
mod string;

use bytes::Bytes;

use crate::{db::Db, resp::RespValue};

pub fn handle_command(request: RespValue, db: &mut Db) -> RespValue {
    let RespValue::Array(parts) = request else {
        return RespValue::Error("ERR invalid command format".to_string());
    };
    let Some(RespValue::BulkString(command)) = parts.first() else {
        return RespValue::Error("ERR invalid command format".to_string());
    };

    let args = &parts[1..];
    let Some(command) = parse_command_name(command) else {
        let unknown = String::from_utf8_lossy(command).into_owned();
        return RespValue::Error(format!("ERR unknown command: {unknown}"));
    };

    match command {
        "PING" | "ECHO" | "INFO" | "FLUSHALL" | "DBSIZE" | "FLUSHDB" => {
            server::handle(command, args, db)
        }
        "SET" | "GET" | "STRLEN" | "MGET" | "MSET" | "INCR" | "DECR" | "INCRBY" | "APPEND"
        | "GETSET" | "GETDEL" => string::handle(command, args, db),
        "EXPIRE" | "PEXPIRE" | "TTL" | "PTTL" | "PERSIST" => expire::handle(command, args, db),
        "DEL" | "EXISTS" | "TYPE" | "RENAME" | "KEYS" => generic::handle(command, args, db),
        "RPUSH" | "LPUSH" | "LRANGE" | "LPOP" | "RPOP" | "LLEN" | "LINDEX" | "LSET" | "LREM" => {
            list::handle(command, args, db)
        }
        "SADD" | "SINTER" | "SREM" | "SMEMBERS" | "SISMEMBER" | "SCARD" | "SUNION" | "SDIFF" => {
            set::handle(command, args, db)
        }
        "HSET" | "HGET" | "HDEL" | "HEXISTS" | "HGETALL" | "HKEYS" | "HVALS" | "HLEN"
        | "HINCRBY" => hash::handle(command, args, db),
        _ => RespValue::Error(format!("ERR unknown command: {command}")),
    }
}

fn parse_command_name(command: &Bytes) -> Option<&'static str> {
    let bytes = command.as_ref();
    if bytes.eq_ignore_ascii_case(b"PING") {
        Some("PING")
    } else if bytes.eq_ignore_ascii_case(b"ECHO") {
        Some("ECHO")
    } else if bytes.eq_ignore_ascii_case(b"INFO") {
        Some("INFO")
    } else if bytes.eq_ignore_ascii_case(b"FLUSHALL") {
        Some("FLUSHALL")
    } else if bytes.eq_ignore_ascii_case(b"DBSIZE") {
        Some("DBSIZE")
    } else if bytes.eq_ignore_ascii_case(b"FLUSHDB") {
        Some("FLUSHDB")
    } else if bytes.eq_ignore_ascii_case(b"SET") {
        Some("SET")
    } else if bytes.eq_ignore_ascii_case(b"GET") {
        Some("GET")
    } else if bytes.eq_ignore_ascii_case(b"STRLEN") {
        Some("STRLEN")
    } else if bytes.eq_ignore_ascii_case(b"MGET") {
        Some("MGET")
    } else if bytes.eq_ignore_ascii_case(b"MSET") {
        Some("MSET")
    } else if bytes.eq_ignore_ascii_case(b"INCR") {
        Some("INCR")
    } else if bytes.eq_ignore_ascii_case(b"DECR") {
        Some("DECR")
    } else if bytes.eq_ignore_ascii_case(b"INCRBY") {
        Some("INCRBY")
    } else if bytes.eq_ignore_ascii_case(b"APPEND") {
        Some("APPEND")
    } else if bytes.eq_ignore_ascii_case(b"GETSET") {
        Some("GETSET")
    } else if bytes.eq_ignore_ascii_case(b"GETDEL") {
        Some("GETDEL")
    } else if bytes.eq_ignore_ascii_case(b"EXPIRE") {
        Some("EXPIRE")
    } else if bytes.eq_ignore_ascii_case(b"PEXPIRE") {
        Some("PEXPIRE")
    } else if bytes.eq_ignore_ascii_case(b"TTL") {
        Some("TTL")
    } else if bytes.eq_ignore_ascii_case(b"PTTL") {
        Some("PTTL")
    } else if bytes.eq_ignore_ascii_case(b"PERSIST") {
        Some("PERSIST")
    } else if bytes.eq_ignore_ascii_case(b"DEL") {
        Some("DEL")
    } else if bytes.eq_ignore_ascii_case(b"EXISTS") {
        Some("EXISTS")
    } else if bytes.eq_ignore_ascii_case(b"TYPE") {
        Some("TYPE")
    } else if bytes.eq_ignore_ascii_case(b"RENAME") {
        Some("RENAME")
    } else if bytes.eq_ignore_ascii_case(b"KEYS") {
        Some("KEYS")
    } else if bytes.eq_ignore_ascii_case(b"RPUSH") {
        Some("RPUSH")
    } else if bytes.eq_ignore_ascii_case(b"LPUSH") {
        Some("LPUSH")
    } else if bytes.eq_ignore_ascii_case(b"LRANGE") {
        Some("LRANGE")
    } else if bytes.eq_ignore_ascii_case(b"LPOP") {
        Some("LPOP")
    } else if bytes.eq_ignore_ascii_case(b"RPOP") {
        Some("RPOP")
    } else if bytes.eq_ignore_ascii_case(b"LLEN") {
        Some("LLEN")
    } else if bytes.eq_ignore_ascii_case(b"LINDEX") {
        Some("LINDEX")
    } else if bytes.eq_ignore_ascii_case(b"LSET") {
        Some("LSET")
    } else if bytes.eq_ignore_ascii_case(b"LREM") {
        Some("LREM")
    } else if bytes.eq_ignore_ascii_case(b"SADD") {
        Some("SADD")
    } else if bytes.eq_ignore_ascii_case(b"SINTER") {
        Some("SINTER")
    } else if bytes.eq_ignore_ascii_case(b"SREM") {
        Some("SREM")
    } else if bytes.eq_ignore_ascii_case(b"SMEMBERS") {
        Some("SMEMBERS")
    } else if bytes.eq_ignore_ascii_case(b"SISMEMBER") {
        Some("SISMEMBER")
    } else if bytes.eq_ignore_ascii_case(b"SCARD") {
        Some("SCARD")
    } else if bytes.eq_ignore_ascii_case(b"SUNION") {
        Some("SUNION")
    } else if bytes.eq_ignore_ascii_case(b"SDIFF") {
        Some("SDIFF")
    } else if bytes.eq_ignore_ascii_case(b"HSET") {
        Some("HSET")
    } else if bytes.eq_ignore_ascii_case(b"HGET") {
        Some("HGET")
    } else if bytes.eq_ignore_ascii_case(b"HDEL") {
        Some("HDEL")
    } else if bytes.eq_ignore_ascii_case(b"HEXISTS") {
        Some("HEXISTS")
    } else if bytes.eq_ignore_ascii_case(b"HGETALL") {
        Some("HGETALL")
    } else if bytes.eq_ignore_ascii_case(b"HKEYS") {
        Some("HKEYS")
    } else if bytes.eq_ignore_ascii_case(b"HVALS") {
        Some("HVALS")
    } else if bytes.eq_ignore_ascii_case(b"HLEN") {
        Some("HLEN")
    } else if bytes.eq_ignore_ascii_case(b"HINCRBY") {
        Some("HINCRBY")
    } else {
        None
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
