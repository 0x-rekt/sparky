use std::time::{Duration, Instant};

use crate::{db::SharedDb, resp::RespValue};

use super::{get_nonnegative_integer, get_string_arg};

pub(super) fn handle(command: &str, args: &[RespValue], db: SharedDb) -> RespValue {
    match command {
        "EXPIRE" => set_expiry(args, db, false),
        "PEXPIRE" => set_expiry(args, db, true),
        "TTL" => get_ttl(args, db, false),
        "PTTL" => get_ttl(args, db, true),
        _ => unreachable!(),
    }
}

fn set_expiry(args: &[RespValue], db: SharedDb, milliseconds: bool) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }
    let key = match get_string_arg(args, 0, "expire") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    let amount = match get_nonnegative_integer(args, 1) {
        Ok(amount) => amount,
        Err(error) => return error,
    };
    let duration = if milliseconds {
        Duration::from_millis(amount)
    } else {
        Duration::from_secs(amount)
    };
    let updated = db
        .lock()
        .unwrap()
        .set_expiry(&key, Instant::now() + duration);
    RespValue::Integer(updated as i64)
}

fn get_ttl(args: &[RespValue], db: SharedDb, milliseconds: bool) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }
    let key = match get_string_arg(args, 0, "ttl") {
        Ok(key) => String::from_utf8_lossy(&key).into_owned(),
        Err(error) => return error,
    };
    RespValue::Integer(db.lock().unwrap().ttl(&key, milliseconds))
}
