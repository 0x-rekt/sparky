use std::collections::HashSet;

use super::get_string_arg;
use crate::{
    db::{Db, SharedDb},
    resp::RespValue,
};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "SADD" => sadd(args, db),
        "SINTER" => sinter(args, db),
        "SREM" => srem(args, db),
        "SMEMBERS" => smembers(args, db),
        "SISMEMBER" => sismember(args, db),
        "SCARD" => scard(args, db),
        "SUNION" => sunion(args, db),
        "SDIFF" => sdiff(args, db),
        _ => unreachable!(),
    }
}

fn sadd(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'sadd' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SADD") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if database.contains_key(&key) && !database.sets.contains_key(&key) {
        return RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        );
    }

    let set = database.sets.entry(key).or_default();

    let mut added_count = 0;
    for arg in &args[1..] {
        if let RespValue::BulkString(value) = arg {
            if set.insert(value.clone()) {
                added_count += 1;
            }
        } else {
            return RespValue::Error("ERR wrong type of argument for 'sadd' command".to_string());
        }
    }

    RespValue::Integer(added_count)
}

fn sinter(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sinter' command".to_string());
    }

    let mut database = db.lock().unwrap();
    let mut intersection: Option<HashSet<bytes::Bytes>> = None;

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SINTER") {
            Ok(key) => String::from_utf8_lossy(&key).into_owned(),
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(&mut database, &key) {
            return error;
        }

        let Some(set) = database.sets.get(&key) else {
            return RespValue::Array(vec![]);
        };
        if let Some(current) = &mut intersection {
            current.retain(|value| set.contains(value));
        } else {
            intersection = Some(set.clone());
        }
    }

    RespValue::Array(
        intersection
            .unwrap_or_default()
            .into_iter()
            .map(RespValue::BulkString)
            .collect(),
    )
}

fn srem(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'srem' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SREM") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if let Err(error) = ensure_set(&mut database, &key) {
        return error;
    }

    let Some(set) = database.sets.get_mut(&key) else {
        return RespValue::Integer(0);
    };

    let mut removed_count = 0;
    for arg in &args[1..] {
        if let RespValue::BulkString(value) = arg {
            if set.remove(value) {
                removed_count += 1;
            }
        } else {
            return RespValue::Error("ERR wrong type of argument for 'srem' command".to_string());
        }
    }

    RespValue::Integer(removed_count)
}

fn smembers(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error(
            "ERR wrong number of arguments for 'smembers' command".to_string(),
        );
    }

    let key = match get_string_arg(args, 0, "SMEMBERS") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if let Err(error) = ensure_set(&mut database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Array(vec![]);
    };

    RespValue::Array(set.iter().cloned().map(RespValue::BulkString).collect())
}

fn sismember(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error(
            "ERR wrong number of arguments for 'sismember' command".to_string(),
        );
    }

    let key = match get_string_arg(args, 0, "SISMEMBER") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let value = match get_string_arg(args, 1, "SISMEMBER") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if let Err(error) = ensure_set(&mut database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Integer(0);
    };

    RespValue::Integer(if set.contains(&value) { 1 } else { 0 })
}

fn scard(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'scard' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SCARD") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if let Err(error) = ensure_set(&mut database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Integer(0);
    };

    RespValue::Integer(set.len() as i64)
}

fn sunion(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sunion' command".to_string());
    }

    let mut database = db.lock().unwrap();
    let mut union_set: HashSet<bytes::Bytes> = HashSet::new();

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SUNION") {
            Ok(key) => String::from_utf8_lossy(&key).into_owned(),
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(&mut database, &key) {
            return error;
        }

        if let Some(set) = database.sets.get(&key) {
            union_set.extend(set.iter().cloned());
        }
    }

    RespValue::Array(union_set.into_iter().map(RespValue::BulkString).collect())
}

fn sdiff(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sdiff' command".to_string());
    }

    let mut database = db.lock().unwrap();
    let mut diff_set: Option<HashSet<bytes::Bytes>> = None;

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SDIFF") {
            Ok(key) => String::from_utf8_lossy(&key).into_owned(),
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(&mut database, &key) {
            return error;
        }

        if let Some(set) = database.sets.get(&key) {
            if let Some(current_diff) = &mut diff_set {
                current_diff.retain(|value| !set.contains(value));
            } else {
                diff_set = Some(set.clone());
            }
        }
    }

    RespValue::Array(
        diff_set
            .unwrap_or_default()
            .into_iter()
            .map(RespValue::BulkString)
            .collect(),
    )
}

fn ensure_set(database: &mut Db, key: &String) -> Result<(), RespValue> {
    if database.contains_key(key)
        && (database.strings.contains_key(key)
            || database.lists.contains_key(key)
            || database.hashes.contains_key(key))
    {
        return Err(RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        ));
    }
    Ok(())
}
