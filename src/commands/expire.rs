use std::time::{Duration, Instant};

use crate::{db::Db, resp::RespValue};

use super::{get_nonnegative_integer, get_string_arg};

pub(super) fn handle(command: &str, args: &[RespValue], db: &mut Db) -> RespValue {
    match command {
        "EXPIRE" => set_expiry(args, db, false),
        "PEXPIRE" => set_expiry(args, db, true),
        "TTL" => get_ttl(args, db, false),
        "PTTL" => get_ttl(args, db, true),
        "PERSIST" => persist(args, db),
        _ => unreachable!(),
    }
}

fn persist(args: &[RespValue], db: &mut Db) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments for 'persist' command".to_string());
    }
    let key = match get_string_arg(args, 0, "persist") {
        Ok(key) => key,
        Err(error) => return error,
    };
    RespValue::Integer(db.persist(&key) as i64)
}

fn set_expiry(args: &[RespValue], db: &mut Db, milliseconds: bool) -> RespValue {
    if args.len() != 2 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }
    let key = match get_string_arg(args, 0, "expire") {
        Ok(key) => key,
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
    let updated = db.set_expiry(&key, Instant::now() + duration);
    RespValue::Integer(updated as i64)
}

fn get_ttl(args: &[RespValue], db: &mut Db, milliseconds: bool) -> RespValue {
    if args.len() != 1 {
        return RespValue::Error("ERR wrong number of arguments".to_string());
    }
    let key = match get_string_arg(args, 0, "ttl") {
        Ok(key) => key,
        Err(error) => return error,
    };
    RespValue::Integer(db.ttl(&key, milliseconds))
}
