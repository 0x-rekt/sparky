use crate::{db::SharedDb, resp::RespValue};

pub async fn handle_command(request: RespValue, db: SharedDb) -> RespValue {
    let RespValue::Array(parts) = request else {
        return RespValue::Error("Invalid command format".to_string());
    };

    let Some(RespValue::BulkString(cmd)) = parts.first() else {
        return RespValue::Error("Invalid command format".to_string());
    };

    let args = &parts[1..];

    match cmd.to_uppercase().as_str() {
        "PING" => {
            if args.len() >= 2 {
                RespValue::Error("Wrong number of arguments for 'PING' command".to_string())
            } else if args.is_empty() {
                RespValue::SimpleString("PONG".to_string())
            } else {
                match args.first() {
                    Some(RespValue::BulkString(message)) => RespValue::BulkString(message.clone()),
                    _ => RespValue::Error("PING command requires a message".to_string()),
                }
            }
        },
        "ECHO" => {
            if let Some(RespValue::BulkString(message)) = args.first() {
                RespValue::BulkString(message.clone())
            } else {
                RespValue::Error("ECHO command requires a message".to_string())
            }
        },
        "SET" => {
            if args.len() != 2 {
                RespValue::Error("Wrong number of arguments for 'SET' command".to_string())
            } else {
                let key = match &args[0] {
                    RespValue::BulkString(k) => k.clone(),
                    _ => return RespValue::Error("SET command requires a string key".to_string()),
                };
                let value = match &args[1] {
                    RespValue::BulkString(v) => v.clone(),
                    _ => return RespValue::Error("SET command requires a string value".to_string()),
                };
                db.lock().unwrap().insert(key, value.into());
                RespValue::SimpleString("OK".to_string())
            }
        }, 
        "GET" => {
            if args.len() != 1 {
                RespValue::Error("Wrong number of arguments for 'GET' command".to_string())
            } else {
                let key = match &args[0] {
                    RespValue::BulkString(k) => k.clone(),
                    _ => return RespValue::Error("GET command requires a string key".to_string()),
                };
                match db.lock().unwrap().get(&key) {
                    Some(value) => RespValue::BulkString(String::from_utf8_lossy(value).to_string()),
                    None => RespValue::Nil,
                }
            }
        },
        _ => RespValue::Error(format!("Unknown command: {}", cmd)),
    }
}