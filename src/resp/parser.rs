use super::RespValue;
use bytes::{Bytes, BytesMut};

pub fn parse_message(buffer: &BytesMut) -> Result<(RespValue, usize), String> {
    parse_message_at(buffer, 0)
}

fn parse_message_at(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (value, end) = match buffer.get(start) {
        Some(&b'+') => parse_simple_string(buffer, start),
        Some(&b'-') => parse_error(buffer, start),
        Some(&b'$') => parse_bulk_string(buffer, start),
        Some(&b':') => parse_integer(buffer, start),
        Some(&b'*') => parse_array(buffer, start),
        _ => Err("Invalid RESP message".to_string()),
    }?;
    Ok((value, end - start))
}

fn parse_simple_string(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (string, end) = read_until_crlf(buffer, start + 1)?;
    let string = String::from_utf8(string.to_vec()).map_err(|e| e.to_string())?;
    Ok((RespValue::SimpleString(string), end))
}

fn parse_error(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (error, end) = read_until_crlf(buffer, start + 1)?;
    let error = String::from_utf8(error.to_vec()).map_err(|e| e.to_string())?;
    Ok((RespValue::Error(error), end))
}

fn parse_bulk_string(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (length_bytes, payload_start) = read_until_crlf(buffer, start + 1)?;
    let length_str = std::str::from_utf8(length_bytes).map_err(|_| "Invalid bulk string length")?;
    let length: i64 = length_str
        .parse()
        .map_err(|_| "Invalid bulk string length".to_string())?;

    if length == -1 {
        return Ok((RespValue::Nil, payload_start));
    }
    let length = length as usize;

    if buffer.len() < payload_start + length + 2 {
        return Err("Incomplete bulk string".to_string());
    }
    let string = Bytes::copy_from_slice(&buffer[payload_start..payload_start + length]);
    Ok((RespValue::BulkString(string), payload_start + length + 2))
}

fn parse_integer(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (int_bytes, end) = read_until_crlf(buffer, start + 1)?;
    let int_str = std::str::from_utf8(int_bytes).map_err(|_| "Invalid integer value")?;
    let int_value: i64 = int_str
        .parse()
        .map_err(|_| "Invalid integer value".to_string())?;
    Ok((RespValue::Integer(int_value), end))
}

fn parse_array(buffer: &[u8], start: usize) -> Result<(RespValue, usize), String> {
    let (count_bytes, mut pos) = read_until_crlf(buffer, start + 1)?;
    let count_str = std::str::from_utf8(count_bytes).map_err(|_| "Invalid array length")?;
    let count: i64 = count_str
        .parse()
        .map_err(|_| "Invalid array length".to_string())?;

    if count == -1 {
        return Ok((RespValue::Nil, pos));
    }

    let mut elements = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if pos >= buffer.len() {
            return Err("Incomplete array".to_string());
        }
        let (value, consumed) = parse_message_at(buffer, pos)?;
        elements.push(value);
        pos += consumed;
    }

    Ok((RespValue::Array(elements), pos))
}

fn read_until_crlf(buffer: &[u8], start: usize) -> Result<(&[u8], usize), String> {
    for i in start..buffer.len() {
        if buffer[i] == b'\r' && buffer.get(i + 1) == Some(&b'\n') {
            return Ok((&buffer[start..i], i + 2));
        }
    }
    Err("Invalid RESP message".to_string())
}
