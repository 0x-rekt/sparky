use bytes::Bytes;

use crate::{db::Db, resp::RespValue};

use super::get_string_arg;

pub(super) fn handle(command: &str, args: &[RespValue], db: &mut Db) -> RespValue {
    match command {
        "DEL" => del(args, db),
        "EXISTS" => exists(args, db),
        "TYPE" => value_type(args, db),
        "RENAME" => rename(args, db),
        "KEYS" => keys(args, db),
        _ => unreachable!(),
    }
}

fn del(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'DEL' command".to_string());
    }
    let mut deleted = 0;
    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "DEL") {
            Ok(key) => key,
            Err(error) => return error,
        };
        deleted += db.remove(&key) as i64;
    }
    RespValue::Integer(deleted)
}

fn exists(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'EXISTS' command".to_string());
    }
    let mut count = 0;
    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "EXISTS") {
            Ok(key) => key,
            Err(error) => return error,
        };
        if db.contains_key(&key) {
            count += 1;
        }
    }
    RespValue::Integer(count)
}

fn value_type(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'TYPE' command".to_string());
    }
    let key = match get_string_arg(args, 0, "TYPE") {
        Ok(key) => key,
        Err(error) => return error,
    };
    let database = db;
    let value_type = if !database.contains_key(&key) {
        "none"
    } else if database.strings.contains_key(&key) {
        "string"
    } else if database.lists.contains_key(&key) {
        "list"
    } else if database.hashes.contains_key(&key) {
        "hash"
    } else if database.sets.contains_key(&key) {
        "set"
    } else {
        "none"
    };
    RespValue::SimpleString(value_type.to_string())
}

fn rename(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'RENAME' command".to_string());
    }
    let old_key = match get_string_arg(args, 0, "RENAME") {
        Ok(key) => key,
        Err(error) => return error,
    };
    let new_key = match get_string_arg(args, 1, "RENAME") {
        Ok(key) => key,
        Err(error) => return error,
    };
    if db.rename(&old_key, &new_key) {
        RespValue::SimpleString("OK".to_string())
    } else {
        RespValue::Error("ERR no such key".to_string())
    }
}

fn keys(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'KEYS' command".to_string());
    }

    let pattern = match get_string_arg(args, 0, "KEYS") {
        Ok(pattern) => String::from_utf8_lossy(&pattern).into_owned(),
        Err(error) => return error,
    };

    let get_all = pattern == "*";
    let prefix = if get_all {
        None
    } else if pattern.ends_with('*') {
        Some(pattern.trim_end_matches('*'))
    } else {
        return RespValue::Error("ERR only supports '*' or 'prefix*' patterns".to_string());
    };

    let database = db;
    let mut keys: Vec<Bytes> = database
        .keys()
        .filter(|key| {
            if get_all {
                true
            } else if let Some(prefix) = prefix {
                key.starts_with(prefix.as_bytes())
            } else {
                false
            }
        })
        .collect();
    keys.sort_unstable();

    let matching_keys = keys.into_iter().map(RespValue::BulkString).collect();

    RespValue::Array(matching_keys)
}
