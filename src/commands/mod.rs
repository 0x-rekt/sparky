use std::time::{Duration, Instant};

use bytes::Bytes;

use crate::{db::SharedDb, resp::RespValue};

pub fn handle_command(request: RespValue, db: SharedDb) -> RespValue {
    let RespValue::Array(parts) = request else {
        return RespValue::Error("ERR invalid command format".to_string());
    };

    let Some(RespValue::BulkString(cmd)) = parts.first() else {
        return RespValue::Error("ERR invalid command format".to_string());
    };

    let args = &parts[1..];
    let cmd = String::from_utf8_lossy(cmd);

    match cmd.to_uppercase().as_str() {
        "PING" => match args.len() {
            0 => RespValue::SimpleString("PONG".to_string()),
            1 => match &args[0] {
                RespValue::BulkString(msg) => RespValue::BulkString(msg.clone()),
                _ => RespValue::Error("ERR value is not a bulk string".to_string()),
            },
            _ => RespValue::Error("ERR wrong number of arguments for 'ping' command".to_string()),
        },
        "ECHO" => match get_string_arg(args, 0, "echo") {
            Ok(msg) => RespValue::BulkString(msg),
            Err(e) => e,
        },
        "SET" => {
            if args.len() < 2 {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'set' command".to_string(),
                );
            }
            let key = match get_string_arg(args, 0, "set") {
                Ok(k) => k,
                Err(e) => return e,
            };
            let value = match get_string_arg(args, 1, "set") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let has_expiry = args.len() > 2
                && matches!(args[2], RespValue::BulkString(ref s) if s.to_ascii_uppercase() == b"EX");

            let expiry = if has_expiry {
                if args.len() != 4 {
                    return RespValue::Error(
                        "ERR wrong number of arguments for 'set' command with expiry".to_string(),
                    );
                }
                let expiry_arg = match get_string_arg(args, 3, "set") {
                    Ok(e) => e,
                    Err(e) => return e,
                };
                let expiry_str = String::from_utf8_lossy(&expiry_arg);
                let expiry_secs: u64 = match expiry_str.parse() {
                    Ok(s) => s,
                    Err(_) => return RespValue::Error("ERR invalid expiry time".to_string()),
                };
                Some(Instant::now() + Duration::from_secs(expiry_secs))
            } else {
                None
            };

            let key = String::from_utf8_lossy(&key).to_string();
            db.lock().unwrap().insert_with_expiry(key, value, expiry);
            RespValue::SimpleString("OK".to_string())
        }
        "GET" => {
            if args.len() != 1 {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'get' command".to_string(),
                );
            }
            let key = match get_string_arg(args, 0, "get") {
                Ok(k) => k,
                Err(e) => return e,
            };
            let key = String::from_utf8_lossy(&key).to_string();
            match db.lock().unwrap().get(&key) {
                Some(value) => RespValue::BulkString(value.clone()),
                None => RespValue::Nil,
            }
        }
        "EXPIRE" => set_expiry(args, db, false),
        "PEXPIRE" => set_expiry(args, db, true),
        "TTL" => get_ttl(args, db, false),
        "PTTL" => get_ttl(args, db, true),
        "DEL" => {
            if args.len() != 1 {
                RespValue::Error("ERR wrong number of arguments for 'DEL' command".to_string())
            } else {
                let key = match get_string_arg(args, 0, "DEL") {
                    Ok(k) => k,
                    Err(e) => return e,
                };
                let key = String::from_utf8_lossy(&key).to_string();
                let removed = db.lock().unwrap().remove(&key);
                if removed.is_some() {
                    RespValue::Integer(1)
                } else {
                    RespValue::Integer(0)
                }
            }
        }
        "EXISTS" => {
            if args.is_empty() {
                RespValue::Error("ERR wrong number of arguments for 'EXISTS' command".to_string())
            } else {
                let mut count = 0;
                for arg in args {
                    let key = match get_string_arg(std::slice::from_ref(arg), 0, "EXISTS") {
                        Ok(k) => k,
                        Err(e) => return e,
                    };
                    let key = String::from_utf8_lossy(&key).to_string();
                    if db.lock().unwrap().contains_key(&key) {
                        count += 1;
                    }
                }
                RespValue::Integer(count)
            }
        }
        "TYPE" => {
            if args.len() != 1 {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'TYPE' command".to_string(),
                );
            }
            let key = match get_string_arg(args, 0, "TYPE") {
                Ok(k) => k,
                Err(e) => return e,
            };
            let key = String::from_utf8_lossy(&key).to_string();
            let mut db_guard = db.lock().unwrap();
            let value_type = if !db_guard.contains_key(&key) {
                "none"
            } else if db_guard.strings.contains_key(&key) {
                "string"
            } else if db_guard.lists.contains_key(&key) {
                "list"
            } else if db_guard.hashes.contains_key(&key) {
                "hash"
            } else if db_guard.sets.contains_key(&key) {
                "set"
            } else {
                "none"
            };
            RespValue::SimpleString(value_type.to_string())
        }
        "STRLEN" => {
            if args.len() != 1 {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'STRLEN' command".to_string(),
                );
            }
            let key = match get_string_arg(args, 0, "STRLEN") {
                Ok(k) => k,
                Err(e) => return e,
            };
            let key = String::from_utf8_lossy(&key).to_string();
            let db_guard = db.lock().unwrap();
            if let Some(value) = db_guard.strings.get(&key) {
                RespValue::Integer(value.len() as i64)
            } else {
                RespValue::Integer(0)
            }
        }
        "MGET" => {
            if args.is_empty() {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'MGET' command".to_string(),
                );
            }
            let mut values = Vec::with_capacity(args.len());
            let db_guard = db.lock().unwrap();
            for arg in args {
                let key = match get_string_arg(std::slice::from_ref(arg), 0, "MGET") {
                    Ok(k) => k,
                    Err(e) => return e,
                };
                let key = String::from_utf8_lossy(&key).to_string();
                if let Some(value) = db_guard.strings.get(&key) {
                    values.push(RespValue::BulkString(value.clone()));
                } else {
                    values.push(RespValue::Nil);
                }
            }
            RespValue::Array(values)
        }
        "MSET" => {
            if args.len() % 2 != 0 || args.is_empty() {
                return RespValue::Error(
                    "ERR wrong number of arguments for 'MSET' command".to_string(),
                );
            }
            let mut db_guard = db.lock().unwrap();
            for i in (0..args.len()).step_by(2) {
                let key = match get_string_arg(args, i, "MSET") {
                    Ok(k) => k,
                    Err(e) => return e,
                };
                let value = match get_string_arg(args, i + 1, "MSET") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let key = String::from_utf8_lossy(&key).to_string();
                db_guard.insert_with_expiry(key, value, None);
            }
            RespValue::SimpleString("OK".to_string())
        }
        _ => RespValue::Error(format!("ERR unknown command: {}", cmd)),
    }
}

fn set_expiry(args: &[RespValue], db: SharedDb, in_millis: bool) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }

    let key = match get_string_arg(args, 0, "expire") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let duration = match get_nonnegative_integer(args, 1) {
        Ok(duration) => duration,
        Err(error) => return error,
    };
    let duration = if in_millis {
        Duration::from_millis(duration)
    } else {
        Duration::from_secs(duration)
    };

    let updated = db
        .lock()
        .unwrap()
        .set_expiry(&key, Instant::now() + duration);
    RespValue::Integer(updated as i64)
}

fn get_ttl(args: &[RespValue], db: SharedDb, in_millis: bool) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }

    let key = match get_string_arg(args, 0, "ttl") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    RespValue::Integer(db.lock().unwrap().ttl(&key, in_millis))
}

fn get_nonnegative_integer(args: &[RespValue], idx: usize) -> Result<u64, RespValue> {
    let value = get_string_arg(args, idx, "expire")
        .map_err(|_| RespValue::Error("ERR invalid expire time".to_string()))?;
    let value = String::from_utf8_lossy(&value);
    value
        .parse::<u64>()
        .map_err(|_| RespValue::Error("ERR invalid expire time".to_string()))
}

fn get_string_arg(args: &[RespValue], idx: usize, cmd: &str) -> Result<Bytes, RespValue> {
    match args.get(idx) {
        Some(RespValue::BulkString(b)) => Ok(b.clone()),
        _ => Err(RespValue::Error(format!(
            "ERR wrong number of arguments for '{}' command",
            cmd.to_lowercase()
        ))),
    }
}
