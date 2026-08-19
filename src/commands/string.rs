use std::time::{Duration, Instant};

use bytes::{Bytes, BytesMut};

use crate::{db::SharedDb, resp::RespValue};

use super::{get_nonnegative_integer, get_string_arg};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "SET" => set(args, db),
        "GET" => get(args, db),
        "STRLEN" => strlen(args, db),
        "MGET" => mget(args, db),
        "MSET" => mset(args, db),
        "INCR" => increment(args, db, 1),
        "DECR" => increment(args, db, -1),
        "INCRBY" => incrby(args, db),
        "APPEND" => append(args, db),
        "GETSET" => getset(args, db),
        "GETDEL" => getdel(args, db),
        _ => unreachable!(),
    }
}

fn set(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'set' command".to_string());
    }
    let key = match get_string_arg(args, 0, "set") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let value = match get_string_arg(args, 1, "set") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let mut expiry = None;
    let mut nx = false;
    let mut xx = false;
    let mut get_old = false;
    let mut index = 2;
    while index < args.len() {
        let option = match get_string_arg(args, index, "set") {
            Ok(option) => option,
            Err(error) => return error,
        };
        let option = String::from_utf8_lossy(&option).to_ascii_uppercase();
        match option.as_str() {
            "EX" | "PX" => {
                if expiry.is_some() || index + 1 >= args.len() {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                let amount = match get_nonnegative_integer(args, index + 1) {
                    Ok(amount) if amount > 0 => amount,
                    _ => {
                        return RespValue::Error(
                            "ERR invalid expire time in 'set' command".to_string(),
                        );
                    }
                };
                expiry = Some(if option == "EX" {
                    Duration::from_secs(amount)
                } else {
                    Duration::from_millis(amount)
                });
                index += 2;
            }
            "NX" => {
                if nx || xx {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                nx = true;
                index += 1;
            }
            "XX" => {
                if nx || xx {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                xx = true;
                index += 1;
            }
            "GET" => {
                if get_old {
                    return RespValue::Error("ERR syntax error".to_string());
                }
                get_old = true;
                index += 1;
            }
            _ => return RespValue::Error("ERR syntax error".to_string()),
        }
    }

    let mut database = db.lock().unwrap();
    let old_value = database.get(&key).cloned();
    let exists = database.contains_key(&key);
    if (nx && exists) || (xx && !exists) {
        return if get_old {
            old_value.map_or(RespValue::Nil, RespValue::BulkString)
        } else {
            RespValue::Nil
        };
    }
    database.insert_with_expiry(key, value, expiry.map(|duration| Instant::now() + duration));
    if get_old {
        old_value.map_or(RespValue::Nil, RespValue::BulkString)
    } else {
        RespValue::SimpleString("OK".to_string())
    }
}

fn get(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'get' command".to_string());
    }
    let key = match get_string_arg(args, 0, "get") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    db.lock()
        .unwrap()
        .get(&key)
        .cloned()
        .map_or(RespValue::Nil, RespValue::BulkString)
}

fn strlen(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'strlen' command".to_string());
    }
    let key = match get_string_arg(args, 0, "strlen") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    RespValue::Integer(db.lock().unwrap().strings.get(&key).map_or(0, Bytes::len) as i64)
}

fn mget(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'mget' command".to_string());
    }
    let database = db.lock().unwrap();
    RespValue::Array(
        args.iter()
            .map(
                |arg| match get_string_arg(std::slice::from_ref(arg), 0, "mget") {
                    Ok(key) => database
                        .strings
                        .get(&String::from_utf8_lossy(&key).into_owned())
                        .cloned()
                        .map_or(RespValue::Nil, RespValue::BulkString),
                    Err(error) => error,
                },
            )
            .collect(),
    )
}

fn mset(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() || !args.len().is_multiple_of(2) {
        return RespValue::Error("ERR wrong number of arguments for 'mset' command".to_string());
    }
    let mut database = db.lock().unwrap();
    for pair in args.chunks_exact(2) {
        let key = match get_string_arg(pair, 0, "mset") {
            Ok(key) => String::from_utf8_lossy(&key).into_owned(),
            Err(error) => return error,
        };
        let value = match get_string_arg(pair, 1, "mset") {
            Ok(value) => value,
            Err(error) => return error,
        };
        database.insert_with_expiry(key, value, None);
    }
    RespValue::SimpleString("OK".to_string())
}

fn increment(args: &[RespValue], db: SharedDb, amount: i64) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }
    let key = match get_string_arg(args, 0, "incr") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
    let current = match database.get(&key) {
        Some(value) => match String::from_utf8_lossy(value).parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        None => 0,
    };
    let value = current + amount;
    database.insert_with_expiry(key, Bytes::from(value.to_string()), None);
    RespValue::Integer(value)
}

fn incrby(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'incrby' command".to_string());
    }
    let increment = match get_string_arg(args, 1, "incrby")
        .ok()
        .and_then(|value| String::from_utf8_lossy(&value).parse::<i64>().ok())
    {
        Some(value) => value,
        None => return RespValue::Error("ERR value is not an integer or out of range".to_string()),
    };
    let key = match get_string_arg(args, 0, "incrby") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
    let current = database
        .get(&key)
        .and_then(|value| String::from_utf8_lossy(value).parse::<i64>().ok())
        .unwrap_or(0);
    let value = match current.checked_add(increment) {
        Some(value) => value,
        None => return RespValue::Error("ERR increment or decrement would overflow".to_string()),
    };
    database.insert_with_expiry(key, Bytes::from(value.to_string()), None);
    RespValue::Integer(value)
}

fn append(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'append' command".to_string());
    }
    let key = match get_string_arg(args, 0, "append") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let append_value = match get_string_arg(args, 1, "append") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
    let mut value = database.get(&key).cloned().unwrap_or_default();
    let mut combined = BytesMut::from(value.as_ref());
    combined.extend_from_slice(&append_value);
    value = combined.freeze();
    let length = value.len() as i64;
    database.insert_with_expiry(key, value, None);
    RespValue::Integer(length)
}

fn getset(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'getset' command".to_string());
    }
    let key = match get_string_arg(args, 0, "getset") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let value = match get_string_arg(args, 1, "getset") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let mut database = db.lock().unwrap();
    let old = database.get(&key).cloned();
    database.insert_with_expiry(key, value, None);
    old.map_or(RespValue::Nil, RespValue::BulkString)
}

fn getdel(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'getdel' command".to_string());
    }
    let key = match get_string_arg(args, 0, "getdel") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    db.lock()
        .unwrap()
        .remove(&key)
        .map_or(RespValue::Nil, RespValue::BulkString)
}
