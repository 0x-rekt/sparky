use crate::{db::Db, resp::RespValue};

pub(super) fn handle(command: &str, args: &[RespValue], db: &mut Db) -> RespValue {
    match command {
        "PING" => match args {
            [] => RespValue::SimpleString("PONG".to_string()),
            [RespValue::BulkString(message)] => RespValue::BulkString(message.clone()),
            _ => RespValue::Error("ERR wrong number of arguments for 'ping' command".to_string()),
        },
        "ECHO" => match args {
            [RespValue::BulkString(message)] => RespValue::BulkString(message.clone()),
            _ => RespValue::Error("ERR wrong number of arguments for 'echo' command".to_string()),
        },
        "FLUSHALL" => match args {
            [] => flushall(args, db),
            _ => {
                RespValue::Error("ERR wrong number of arguments for 'flushall' command".to_string())
            }
        },
        "DBSIZE" => match args {
            [] => dbsize(db),
            _ => RespValue::Error("ERR wrong number of arguments for 'dbsize' command".to_string()),
        },
        "INFO" => match args {
            [] => info(db),
            _ => RespValue::Error("ERR wrong number of arguments for 'info' command".to_string()),
        },
        "FLUSHDB" => match args {
            [] => flushdb(args, db),
            _ => {
                RespValue::Error("ERR wrong number of arguments for 'flushdb' command".to_string())
            }
        },
        _ => unreachable!(),
    }
}

fn flushall(args: &[RespValue], db: &mut Db) -> RespValue {
    if !args.is_empty() {
        return RespValue::Error(
            "ERR wrong number of arguments for 'flushall' command".to_string(),
        );
    }
    db.strings.clear();
    db.lists.clear();
    db.hashes.clear();
    db.sets.clear();
    db.expirations.clear();
    RespValue::SimpleString("OK".to_string())
}

fn dbsize(db: &mut Db) -> RespValue {
    let size = db.strings.len() + db.lists.len() + db.hashes.len() + db.sets.len();
    RespValue::Integer(size as i64)
}

fn info(db: &mut Db) -> RespValue {
    let total_keys = db.strings.len() + db.lists.len() + db.hashes.len() + db.sets.len();
    let info = format!(
        "# Server\nversion: 0.1.0\n# Database\nkeys: {}\n",
        total_keys
    );
    RespValue::BulkString(info.into_bytes().into())
}

fn flushdb(args: &[RespValue], db: &mut Db) -> RespValue {
    if !args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'flushdb' command".to_string());
    }
    db.strings.clear();
    db.lists.clear();
    db.hashes.clear();
    db.sets.clear();
    db.expirations.clear();
    RespValue::SimpleString("OK".to_string())
}
