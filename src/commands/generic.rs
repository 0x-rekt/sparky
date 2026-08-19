use crate::{db::SharedDb, resp::RespValue};

use super::get_string_arg;

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "DEL" => del(args, db),
        "EXISTS" => exists(args, db),
        "TYPE" => value_type(args, db),
        "RENAME" => rename(args, db),
        "KEYS" => keys(args, db),
        _ => unreachable!(),
    }
}

fn del(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'DEL' command".to_string());
    }
    let key = match get_string_arg(args, 0, "DEL") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    RespValue::Integer(db.lock().unwrap().remove(&key).is_some() as i64)
}

fn exists(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'EXISTS' command".to_string());
    }
    let mut count = 0;
    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "EXISTS") {
            Ok(key) => String::from_utf8_lossy(&key).into_owned(),
            Err(error) => return error,
        };
        if db.lock().unwrap().contains_key(&key) {
            count += 1;
        }
    }
    RespValue::Integer(count)
}

fn value_type(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'TYPE' command".to_string());
    }
    let key = match get_string_arg(args, 0, "TYPE") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
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

fn rename(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'RENAME' command".to_string());
    }
    let old_key = match get_string_arg(args, 0, "RENAME") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let new_key = match get_string_arg(args, 1, "RENAME") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
    match database.remove(&old_key) {
        Some(value) => {
            database.insert_with_expiry(new_key, value, None);
            RespValue::SimpleString("OK".to_string())
        }
        None => RespValue::Error("ERR no such key".to_string()),
    }
}

fn keys(args: &[RespValue], db: SharedDb) -> RespValue {
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

    let mut database = db.lock().unwrap();
    let mut keys: Vec<String> = database
        .keys()
        .filter(|key| {
            if get_all {
                true
            } else if let Some(prefix) = prefix {
                key.starts_with(prefix)
            } else {
                false
            }
        })
        .collect();
    keys.sort_unstable();

    let matching_keys = keys
        .into_iter()
        .map(|key| RespValue::BulkString(key.into_bytes().into()))
        .collect();

    RespValue::Array(matching_keys)
}
