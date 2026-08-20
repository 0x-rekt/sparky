use crate::{db::Db, resp::RespValue};
use bytes::Bytes;

use super::get_string_arg;

pub(super) fn handle(command: &str, args: &[RespValue], db: &mut Db) -> RespValue {
    match command {
        "HSET" => hset(args, db),
        "HGET" => hget(args, db),
        "HDEL" => hdel(args, db),
        "HEXISTS" => hexists(args, db),
        "HGETALL" => hgetall(args, db),
        "HKEYS" => hkeys(args, db),
        "HVALS" => hvals(args, db),
        "HLEN" => hlen(args, db),
        "HINCRBY" => hincrby(args, db),
        _ => RespValue::Error(format!("ERR unknown command: {command}")),
    }
}

fn hset(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return RespValue::Error("ERR wrong number of arguments for 'hset' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HSET") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let database = db;

    if let Err(err) = ensure_hash(database, &key) {
        return err;
    }

    let hash = database.hashes.entry(key.clone()).or_default();

    let mut count = 0;
    for i in (1..args.len()).step_by(2) {
        let field = match get_string_arg(args, i, "HSET") {
            Ok(field) => String::from_utf8_lossy(&field).into_owned(),
            Err(error) => return error,
        };
        let value = match get_string_arg(args, i + 1, "HSET") {
            Ok(value) => value,
            Err(error) => return error,
        };

        if hash.insert(field, value).is_none() {
            count += 1;
        }
    }
    RespValue::Integer(count)
}

fn hget(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'hget' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HGET") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let field = match get_string_arg(args, 1, "HGET") {
        Ok(field) => String::from_utf8_lossy(&field).into_owned(),
        Err(error) => return error,
    };

    let database = db;
    if let Some(hash) = database.hashes.get(&key)
        && let Some(value) = hash.get(&field)
    {
        return RespValue::BulkString(value.clone());
    }
    RespValue::Nil
}

fn hdel(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() < 2 {
        return RespValue::Error("ERR wrong number of arguments for 'hdel' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HDEL") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let database = db;
    if let Some(hash) = database.hashes.get_mut(&key) {
        let mut count = 0;
        for i in 1..args.len() {
            let field = match get_string_arg(args, i, "HDEL") {
                Ok(field) => String::from_utf8_lossy(&field).into_owned(),
                Err(error) => return error,
            };
            if hash.remove(&field).is_some() {
                count += 1;
            }
        }

        if hash.is_empty() {
            database.hashes.remove(&key);
        }

        return RespValue::Integer(count);
    }
    RespValue::Integer(0)
}

fn hexists(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments for 'hexists' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HEXISTS") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let field = match get_string_arg(args, 1, "HEXISTS") {
        Ok(field) => String::from_utf8_lossy(&field).into_owned(),
        Err(error) => return error,
    };

    let database = db;
    if let Some(hash) = database.hashes.get(&key)
        && hash.contains_key(&field)
    {
        return RespValue::Integer(1);
    }
    RespValue::Integer(0)
}

fn hkeys(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'hkeys' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HKEYS") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let database = db;
    if let Some(hash) = database.hashes.get(&key) {
        let keys: Vec<RespValue> = hash
            .keys()
            .map(|k| RespValue::BulkString(Bytes::from(k.clone())))
            .collect();
        return RespValue::Array(keys);
    }
    RespValue::Array(vec![])
}

fn hvals(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'hvals' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HVALS") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let database = db;
    if let Some(hash) = database.hashes.get(&key) {
        let vals: Vec<RespValue> = hash
            .values()
            .map(|v| RespValue::BulkString(v.clone()))
            .collect();
        return RespValue::Array(vals);
    }
    RespValue::Array(vec![])
}

fn hlen(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'hlen' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HLEN") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let database = db;
    if let Some(hash) = database.hashes.get(&key) {
        return RespValue::Integer(hash.len() as i64);
    }
    RespValue::Integer(0)
}

fn hincrby(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 3 {
        return RespValue::Error("ERR wrong number of arguments for 'hincrby' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HINCRBY") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let field = match get_string_arg(args, 1, "HINCRBY") {
        Ok(field) => String::from_utf8_lossy(&field).into_owned(),
        Err(error) => return error,
    };

    let increment = match get_string_arg(args, 2, "HINCRBY") {
        Ok(increment) => match String::from_utf8_lossy(&increment).parse::<i64>() {
            Ok(num) => num,
            Err(_) => return RespValue::Error("ERR hash value is not an integer".to_string()),
        },
        Err(error) => return error,
    };

    let database = db;
    let hash = database.hashes.entry(key.clone()).or_default();
    let current_value = match hash.get(&field) {
        None => 0,
        Some(value) => match String::from_utf8_lossy(value).parse::<i64>() {
            Ok(value) => value,
            Err(_) => return RespValue::Error("ERR hash value is not an integer".to_string()),
        },
    };
    let new_value = match current_value.checked_add(increment) {
        Some(value) => value,
        None => return RespValue::Error("ERR increment or decrement would overflow".to_string()),
    };
    hash.insert(field, Bytes::from(new_value.to_string()));
    RespValue::Integer(new_value)
}

fn hgetall(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'hgetall' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HGETALL") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    if let Err(err) = ensure_hash(db, &key) {
        return err;
    }

    let database = db;
    if let Some(hash) = database.hashes.get(&key) {
        let mut result = Vec::with_capacity(hash.len() * 2);
        for (field, value) in hash.iter() {
            result.push(RespValue::BulkString(Bytes::from(field.clone())));
            result.push(RespValue::BulkString(value.clone()));
        }
        return RespValue::Array(result);
    }
    RespValue::Array(vec![])
}

fn ensure_hash(database: &mut Db, key: &String) -> Result<(), RespValue> {
    if database.contains_key(key) && !database.hashes.contains_key(key) {
        return Err(RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        ));
    }
    Ok(())
}
