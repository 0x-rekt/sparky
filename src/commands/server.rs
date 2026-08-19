use crate::resp::RespValue;

pub(super) fn handle(command: &str, args: &[RespValue]) -> RespValue {
    match command {
        "PING" => match args {
            [] => RespValue::SimpleString("PONG".to_string()),
            [RespValue::BulkString(message)] => RespValue::BulkString(message.clone()),
            _ => RespValue::Error("ERR wrong number of arguments for 'ping' command".to_string()),
        },
        "ECHO" => match args {
            [RespValue::BulkString(message)] => RespValue::BulkString(message.clone()),
            _ => RespValue::Error("ERR wrong number of arguments for 'echo' command".to_string()),
        },
        _ => unreachable!(),
    }
}
