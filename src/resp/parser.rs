use bytes::{Bytes, BytesMut};
use super::RespValue;

pub fn parse_message(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    match buffer.first() {
        Some(&b'+') => parse_simple_string(buffer),
        Some(&b'-') => parse_error(buffer),
        Some(&b'$') => parse_bulk_string(buffer),
        Some(&b':') => parse_integer(buffer),
        Some(&b'*') => parse_array(buffer),
        _ => Err("Invalid RESP message".to_string()),
    }
}

fn parse_simple_string(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    let (string, end) = read_until_crlf(buffer, 1)?;
    Ok((RespValue::SimpleString(string), end))
}

fn parse_error(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    let (error, end) = read_until_crlf(buffer, 1)?;
    Ok((RespValue::Error(error), end))
}

fn parse_bulk_string(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    let (length_str, start) = read_until_crlf(buffer, 1)?;
    let length: i64 = length_str.parse().map_err(|_| "Invalid bulk string length".to_string())?;

    if length == -1 {
        return Ok((RespValue::Nil, start));
    }
    let length = length as usize;

    if buffer.len() < start + length + 2 {
        return Err("Incomplete bulk string".to_string());
    }
    let string = Bytes::copy_from_slice(&buffer[start..start + length]);
    Ok((RespValue::BulkString(string), start + length + 2))
}

fn parse_integer(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    let (int_str, end) = read_until_crlf(buffer, 1)?;
    let int_value: i64 = int_str.parse().map_err(|_| "Invalid integer value".to_string())?;
    Ok((RespValue::Integer(int_value), end))
}

fn parse_array(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    let (count_str, mut pos) = read_until_crlf(buffer, 1)?;
    let count: i64 = count_str.parse().map_err(|_| "Invalid array length".to_string())?;

    if count == -1 {
        return Ok((RespValue::Nil, pos));
    }

    let mut elements = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if pos >= buffer.len() {
            return Err("Incomplete array".to_string());
        }
        let remaining = BytesMut::from(&buffer[pos..]);
        let (value, consumed) = parse_message(&remaining)?;
        elements.push(value);
        pos += consumed;
    }

    Ok((RespValue::Array(elements), pos))
}

fn read_until_crlf(buffer: &BytesMut, start: usize) -> Result<(String, usize), String> {
    for i in start..buffer.len() {
        if buffer[i] == b'\r' && buffer.get(i + 1) == Some(&b'\n') {
            let string = String::from_utf8(buffer[start..i].to_vec()).map_err(|e| e.to_string())?;
            return Ok((string, i + 2));
        }
    }
    Err("Invalid RESP message".to_string())
}
