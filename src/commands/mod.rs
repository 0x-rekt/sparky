use crate::resp::RespValue;

pub fn handle_command(request: RespValue) -> RespValue {
    let RespValue::Array(parts) = request else {
        return RespValue::Error("Invalid command format".to_string());
    };

    let Some(RespValue::BulkString(cmd)) = parts.first() else {
        return RespValue::Error("Invalid command format".to_string());
    };

    let args = &parts[1..];

    match cmd.to_uppercase().as_str() {
        "PING" => RespValue::SimpleString("PONG".to_string()),
        "ECHO" => {
            if let Some(RespValue::BulkString(message)) = args.first() {
                RespValue::BulkString(message.clone())
            } else {
                RespValue::Error("ECHO command requires a message".to_string())
            }
        }
        _ => RespValue::Error(format!("Unknown command: {}", cmd)),
    }
}