use super::get_string_arg;
use crate::{db::SharedDb, resp::RespValue};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "RPUSH" => rpush(args, db),
        "LPUSH" => lpush(args, db),
        "LRANGE" => lrange(args, db),
        "LLEN" => llen(args, db),
        "LPOP" => lpop(args, db),
        "RPOP" => rpop(args, db),
        "LINDEX" => lindex(args, db),
        "LSET" => lset(args, db),
        "LREM" => lrem(args, db),
        _ => unreachable!(),
    }
}

fn rpush(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'RPUSH'".to_string());
    }

    let key = match get_string_arg(args, 0, "RPUSH") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if database.strings.contains_key(&key)
        || database.hashes.contains_key(&key)
        || database.sets.contains_key(&key)
    {
        return RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        );
    }

    let list = database.lists.entry(key).or_default();

    for arg in &args[1..] {
        if let RespValue::BulkString(value) = arg {
            list.push_back(value.clone());
        } else {
            return RespValue::Error("ERR invalid argument type for 'RPUSH'".to_string());
        }
    }
    RespValue::Integer(list.len() as i64)
}

fn lpush(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'LPUSH'".to_string());
    }

    let key = match get_string_arg(args, 0, "LPUSH") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if database.strings.contains_key(&key)
        || database.hashes.contains_key(&key)
        || database.sets.contains_key(&key)
    {
        return RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        );
    }

    let list = database.lists.entry(key).or_default();

    for arg in &args[1..] {
        if let RespValue::BulkString(value) = arg {
            list.push_front(value.clone());
        } else {
            return RespValue::Error("ERR invalid argument type for 'LPUSH'".to_string());
        }
    }
    RespValue::Integer(list.len() as i64)
}

fn lrange(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'LRANGE'".to_string());
    }

    let key = match get_string_arg(args, 0, "LRANGE") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let list = {
        let database = db.lock().unwrap();
        if database.strings.contains_key(&key)
            || database.hashes.contains_key(&key)
            || database.sets.contains_key(&key)
        {
            return RespValue::Error(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
            );
        }
        match database.lists.get(&key) {
            Some(list) => list.clone(),
            None => return RespValue::Array(vec![]),
        }
    };

    let start = match get_string_arg(args, 1, "LRANGE") {
        Ok(start) => match String::from_utf8_lossy(&start).parse::<isize>() {
            Ok(start) => start,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        Err(error) => return error,
    };

    let stop = match get_string_arg(args, 2, "LRANGE") {
        Ok(stop) => match String::from_utf8_lossy(&stop).parse::<isize>() {
            Ok(stop) => stop,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        Err(error) => return error,
    };

    let length = list.len() as isize;
    let start = if start < 0 {
        (length + start).max(0)
    } else {
        start
    };
    let stop = if stop < 0 {
        (length + stop).max(0)
    } else {
        stop
    };

    if start >= length || start > stop {
        return RespValue::Array(vec![]);
    }

    let stop = stop.min(length - 1);
    RespValue::Array(
        list.into_iter()
            .skip(start as usize)
            .take((stop - start + 1) as usize)
            .map(RespValue::BulkString)
            .collect(),
    )
}

fn llen(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'LLEN'".to_string());
    }

    let key = match get_string_arg(args, 0, "LLEN") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let database = db.lock().unwrap();

    if database.strings.contains_key(&key)
        || database.hashes.contains_key(&key)
        || database.sets.contains_key(&key)
    {
        return RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        );
    }

    let length = database.lists.get(&key).map_or(0, |list| list.len());
    RespValue::Integer(length as i64)
}

fn lpop(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() || args.len() > 2 {
        return RespValue::Error("ERR wrong number of arguments for 'LPOP'".to_string());
    }

    let key = match get_string_arg(args, 0, "LPOP") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let with_count = args.len() == 2;
    let count = if with_count {
        match get_string_arg(args, 1, "LPOP") {
            Ok(count) => match String::from_utf8_lossy(&count).parse::<usize>() {
                Ok(count) => count,
                Err(_) => {
                    return RespValue::Error(
                        "ERR value is not an integer or out of range".to_string(),
                    );
                }
            },
            Err(error) => return error,
        }
    } else {
        1
    };

    let mut database = db.lock().unwrap();
    let list = match database.lists.get_mut(&key) {
        Some(list) => list,
        None => {
            return if with_count {
                RespValue::Array(vec![])
            } else {
                RespValue::Nil
            };
        }
    };

    let mut removed_elements = Vec::new();
    for _ in 0..count {
        if let Some(value) = list.pop_front() {
            removed_elements.push(RespValue::BulkString(value));
        } else {
            break;
        }
    }

    if list.is_empty() {
        database.lists.remove(&key);
    }

    if with_count {
        RespValue::Array(removed_elements)
    } else {
        removed_elements
            .into_iter()
            .next()
            .unwrap_or(RespValue::Nil)
    }
}

fn rpop(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() || args.len() > 2 {
        return RespValue::Error("ERR wrong number of arguments for 'RPOP'".to_string());
    }

    let key = match get_string_arg(args, 0, "RPOP") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let with_count = args.len() == 2;
    let count = if with_count {
        match get_string_arg(args, 1, "RPOP") {
            Ok(count) => match String::from_utf8_lossy(&count).parse::<usize>() {
                Ok(count) => count,
                Err(_) => {
                    return RespValue::Error(
                        "ERR value is not an integer or out of range".to_string(),
                    );
                }
            },
            Err(error) => return error,
        }
    } else {
        1
    };

    let mut database = db.lock().unwrap();
    let list = match database.lists.get_mut(&key) {
        Some(list) => list,
        None => {
            return if with_count {
                RespValue::Array(vec![])
            } else {
                RespValue::Nil
            };
        }
    };

    let mut removed_elements = Vec::new();
    for _ in 0..count {
        if let Some(value) = list.pop_back() {
            removed_elements.push(RespValue::BulkString(value));
        } else {
            break;
        }
    }

    if list.is_empty() {
        database.lists.remove(&key);
    }

    if with_count {
        RespValue::Array(removed_elements)
    } else {
        removed_elements
            .into_iter()
            .next()
            .unwrap_or(RespValue::Nil)
    }
}

fn lindex(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'LINDEX'".to_string());
    }

    let key = match get_string_arg(args, 0, "LINDEX") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let index = match get_string_arg(args, 1, "LINDEX") {
        Ok(index) => match String::from_utf8_lossy(&index).parse::<isize>() {
            Ok(index) => index,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        Err(error) => return error,
    };

    let database = db.lock().unwrap();
    let list = match database.lists.get(&key) {
        Some(list) => list,
        None => return RespValue::Nil,
    };

    let length = list.len() as isize;
    let index = if index < 0 {
        (length + index).max(0)
    } else {
        index
    };

    if index < 0 || index >= length {
        return RespValue::Nil;
    }

    RespValue::BulkString(list[index as usize].clone())
}

fn lset(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'LSET'".to_string());
    }

    let key = match get_string_arg(args, 0, "LSET") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let index = match get_string_arg(args, 1, "LSET") {
        Ok(index) => match String::from_utf8_lossy(&index).parse::<isize>() {
            Ok(index) => index,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        Err(error) => return error,
    };

    let value = match get_string_arg(args, 2, "LSET") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();
    let list = match database.lists.get_mut(&key) {
        Some(list) => list,
        None => return RespValue::Error("ERR no such key".to_string()),
    };

    let length = list.len() as isize;
    let index = if index < 0 {
        (length + index).max(0)
    } else {
        index
    };

    if index < 0 || index >= length {
        return RespValue::Error("ERR index out of range".to_string());
    }

    list[index as usize] = value.clone();
    RespValue::SimpleString("OK".to_string())
}

fn lrem(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 3 {
        return RespValue::Error("ERR wrong number of arguments for 'LREM'".to_string());
    }

    let key = match get_string_arg(args, 0, "LREM") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let count = match get_string_arg(args, 1, "LREM") {
        Ok(count) => match String::from_utf8_lossy(&count).parse::<isize>() {
            Ok(count) => count,
            Err(_) => {
                return RespValue::Error("ERR value is not an integer or out of range".to_string());
            }
        },
        Err(error) => return error,
    };

    let value = match get_string_arg(args, 2, "LREM") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();
    let list = match database.lists.get_mut(&key) {
        Some(list) => list,
        None => return RespValue::Integer(0),
    };

    let original_length = list.len();
    if count == 0 {
        list.retain(|x| x != &value);
    } else if count > 0 {
        let mut removed = 0;
        list.retain(|x| {
            if x == &value && removed < count as usize {
                removed += 1;
                false
            } else {
                true
            }
        });
    } else {
        let mut removed = 0;
        list.make_contiguous().reverse();
        list.retain(|x| {
            if x == &value && removed < (-count) as usize {
                removed += 1;
                false
            } else {
                true
            }
        });
        list.make_contiguous().reverse();
    }

    let removed_count = original_length - list.len();
    RespValue::Integer(removed_count as i64)
}
