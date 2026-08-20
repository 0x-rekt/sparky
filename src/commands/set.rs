use super::get_string_arg;
use crate::{db::SharedDb, resp::RespValue};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "SADD" => sadd(args, db),
        "SINTER" => sinter(args, db),
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
    if args.is_empty() ||  args.len() != 2{
        return RespValue::Error("ERR wrong number of arguments for 'sinter' command".to_string());
    }

    let key1 = match get_string_arg(args, 0, "SINTER") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let key2 = match get_string_arg(args, 1, "SINTER") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let database = db.lock().unwrap();

    let set1 = database.sets.get(&key1);
    let set2 = database.sets.get(&key2);

    let intersection: Vec<RespValue> = match (set1, set2) {
        (Some(s1), Some(s2)) => s1
            .intersection(s2)
            .map(|value| RespValue::BulkString(value.clone()))
            .collect(),
        _ => Vec::new(),
    };

    RespValue::Array(intersection)

}