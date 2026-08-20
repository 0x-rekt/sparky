use crate::{
    db::{Db, SharedDb},
    resp::RespValue,
};

use super::get_string_arg;

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "HSET" => hset(args, db),
        // "HGET" => hget(args, db),
        // "HDEL" => hdel(args, db),
        // "HEXISTS" => hexists(args, db),
        // "HGETALL" => hgetall(args, db),
        _ => RespValue::Error(format!("ERR unknown command: {command}")),
    }
}

// HSET key field value [field value ...]
fn hset(args: &[RespValue], db: SharedDb) -> RespValue {
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return RespValue::Error("ERR wrong number of arguments for 'hset' command".to_string());
    }

    let key = match get_string_arg(args, 0, "HSET") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };

    let mut database = db.lock().unwrap();

    if let Err(err) = ensure_hash(&mut database, &key) {
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

fn ensure_hash(database: &mut Db, key: &String) -> Result<(), RespValue> {
    if database.contains_key(key) && !database.hashes.contains_key(key) {
        return Err(RespValue::Error(
            "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
        ));
    }
    Ok(())
}
