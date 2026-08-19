use super::get_string_arg;
use crate::{db::SharedDb, resp::RespValue};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "RPUSH" => rpush(args, db),
        "LPUSH" => lpush(args, db),
        "LRANGE" => lrange(args, db),
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