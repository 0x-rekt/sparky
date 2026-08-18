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
            if args.len() != 2 {
                return RespValue::Error("ERR wrong number of arguments for 'set' command".to_string());
            }
            let key = match get_string_arg(args, 0, "set") {
                Ok(k) => k,
                Err(e) => return e,
            };
            let value = match get_string_arg(args, 1, "set") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let key = String::from_utf8_lossy(&key).to_string();
            db.lock().unwrap().insert(key, value);
            RespValue::SimpleString("OK".to_string())
        }
        "GET" => {
            if args.len() != 1 {
                return RespValue::Error("ERR wrong number of arguments for 'get' command".to_string());
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
        },
        "DEL" => {
            if args.len() != 1 {
                return RespValue::Error("ERR wrong number of arguments for 'DEL' command".to_string())
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
        _ => RespValue::Error(format!("ERR unknown command: {}", cmd)),
    }
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
