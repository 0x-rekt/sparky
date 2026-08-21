use std::collections::HashSet;

use super::get_string_arg;
use crate::{db::Db, resp::RespValue};

pub(super) fn handle(command: &str, args: &[RespValue], db: &mut Db) -> RespValue {
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

fn sadd(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'sadd' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SADD") {
        Ok(key) => key,
        Err(error) => return error,
    };

    let database = db;

    if database.has_wrong_type_for_set(&key) {
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

fn sinter(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sinter' command".to_string());
    }

    let database = db;
    let mut intersection: Option<HashSet<bytes::Bytes>> = None;

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SINTER") {
            Ok(key) => key,
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(database, &key) {
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

fn srem(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'srem' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SREM") {
        Ok(key) => key,
        Err(error) => return error,
    };

    let database = db;

    if let Err(error) = ensure_set(database, &key) {
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

fn smembers(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error(
            "ERR wrong number of arguments for 'smembers' command".to_string(),
        );
    }

    let key = match get_string_arg(args, 0, "SMEMBERS") {
        Ok(key) => key,
        Err(error) => return error,
    };

    let database = db;

    if let Err(error) = ensure_set(database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Array(vec![]);
    };

    RespValue::Array(set.iter().cloned().map(RespValue::BulkString).collect())
}

fn sismember(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error(
            "ERR wrong number of arguments for 'sismember' command".to_string(),
        );
    }

    let key = match get_string_arg(args, 0, "SISMEMBER") {
        Ok(key) => key,
        Err(error) => return error,
    };

    let value = match get_string_arg(args, 1, "SISMEMBER") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let database = db;

    if let Err(error) = ensure_set(database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Integer(0);
    };

    RespValue::Integer(if set.contains(&value) { 1 } else { 0 })
}

fn scard(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'scard' command".to_string());
    }

    let key = match get_string_arg(args, 0, "SCARD") {
        Ok(key) => key,
        Err(error) => return error,
    };

    let database = db;

    if let Err(error) = ensure_set(database, &key) {
        return error;
    }

    let Some(set) = database.sets.get(&key) else {
        return RespValue::Integer(0);
    };

    RespValue::Integer(set.len() as i64)
}

fn sunion(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sunion' command".to_string());
    }

    let database = db;
    let mut union_set: HashSet<bytes::Bytes> = HashSet::new();

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SUNION") {
            Ok(key) => key,
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(database, &key) {
            return error;
        }

        if let Some(set) = database.sets.get(&key) {
            union_set.extend(set.iter().cloned());
        }
    }

    RespValue::Array(union_set.into_iter().map(RespValue::BulkString).collect())
}

fn sdiff(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.is_empty() {
        return RespValue::Error("ERR wrong number of arguments for 'sdiff' command".to_string());
    }

    let database = db;
    let mut diff_set: Option<HashSet<bytes::Bytes>> = None;

    for arg in args {
        let key = match get_string_arg(std::slice::from_ref(arg), 0, "SDIFF") {
            Ok(key) => key,
            Err(error) => return error,
        };
        if let Err(error) = ensure_set(database, &key) {
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

fn ensure_set(database: &mut Db, key: &[u8]) -> Result<(), RespValue> {
    if database.has_wrong_type_for_set(key) {
        return Err(RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        ));
    }
    Ok(())
}
