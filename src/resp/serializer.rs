use super::RespValue;

pub fn serialize_into(value: &RespValue, out: &mut Vec<u8>) {
    match value {
        RespValue::SimpleString(s) => {
            out.push(b'+');
            out.extend_from_slice(s.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        RespValue::Error(e) => {
            out.push(b'-');
            out.extend_from_slice(e.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        RespValue::Integer(i) => {
            out.push(b':');
            out.extend_from_slice(i.to_string().as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        RespValue::BulkString(s) => {
            out.push(b'$');
            out.extend_from_slice(s.len().to_string().as_bytes());
            out.extend_from_slice(b"\r\n");
            out.extend_from_slice(s);
            out.extend_from_slice(b"\r\n");
        }
        RespValue::Nil => out.extend_from_slice(b"$-1\r\n"),
        RespValue::Array(items) => {
            out.push(b'*');
            out.extend_from_slice(items.len().to_string().as_bytes());
            out.extend_from_slice(b"\r\n");
            for item in items {
                serialize_into(item, out);
            }
        }
    }
}
