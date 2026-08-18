use super::RespValue;

pub fn serialize(value: &RespValue) -> Vec<u8> {
    match value {
        RespValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
        RespValue::Error(e) => format!("-{}\r\n", e).into_bytes(),
        RespValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
        RespValue::BulkString(s) => {
            let mut out = format!("${}\r\n", s.len()).into_bytes();
            out.extend_from_slice(s);
            out.extend_from_slice(b"\r\n");
            out
        }
        RespValue::Nil => b"$-1\r\n".to_vec(),
        RespValue::Array(items) => {
            let mut out = format!("*{}\r\n", items.len()).into_bytes();
            for item in items {
                out.extend(serialize(item));
            }
            out
        }
    }
}
