use crate::error::{BittorrentError, Result};
use super::BencodeValue;
use std::collections::BTreeMap;

/// Decode bencoded data into a BencodeValue
pub fn decode(data: &[u8]) -> Result<BencodeValue> {
    let mut pos = 0;
    decode_value(data, &mut pos)
}

fn decode_value(data: &[u8], pos: &mut usize) -> Result<BencodeValue> {
    if *pos >= data.len() {
        return Err(BittorrentError::BencodeError(
            "Unexpected end of input".to_string(),
        ));
    }

    match data[*pos] {
        b'i' => decode_integer(data, pos),
        b'l' => decode_list(data, pos),
        b'd' => decode_dict(data, pos),
        b'0'..=b'9' => decode_string(data, pos),
        c => Err(BittorrentError::BencodeError(format!(
            "Invalid bencode token: {}",
            c as char
        ))),
    }
}

fn decode_integer(data: &[u8], pos: &mut usize) -> Result<BencodeValue> {
    *pos += 1; // Skip 'i'

    let start = *pos;
    while *pos < data.len() && data[*pos] != b'e' {
        *pos += 1;
    }

    if *pos >= data.len() {
        return Err(BittorrentError::BencodeError(
            "Unterminated integer".to_string(),
        ));
    }

    let num_str = std::str::from_utf8(&data[start..*pos])
        .map_err(|_| BittorrentError::BencodeError("Invalid integer".to_string()))?;

    let num = num_str
        .parse::<i64>()
        .map_err(|_| BittorrentError::BencodeError("Invalid integer".to_string()))?;

    *pos += 1; // Skip 'e'

    Ok(BencodeValue::Integer(num))
}

fn decode_string(data: &[u8], pos: &mut usize) -> Result<BencodeValue> {
    let start = *pos;
    while *pos < data.len() && data[*pos] != b':' {
        *pos += 1;
    }

    if *pos >= data.len() {
        return Err(BittorrentError::BencodeError(
            "Invalid string length".to_string(),
        ));
    }

    let len_str = std::str::from_utf8(&data[start..*pos])
        .map_err(|_| BittorrentError::BencodeError("Invalid string length".to_string()))?;

    let len = len_str
        .parse::<usize>()
        .map_err(|_| BittorrentError::BencodeError("Invalid string length".to_string()))?;

    *pos += 1; // Skip ':'

    if *pos + len > data.len() {
        return Err(BittorrentError::BencodeError(
            "String length exceeds data".to_string(),
        ));
    }

    let string = data[*pos..*pos + len].to_vec();
    *pos += len;

    Ok(BencodeValue::String(string))
}

fn decode_list(data: &[u8], pos: &mut usize) -> Result<BencodeValue> {
    *pos += 1; // Skip 'l'

    let mut list = Vec::new();

    while *pos < data.len() && data[*pos] != b'e' {
        list.push(decode_value(data, pos)?);
    }

    if *pos >= data.len() {
        return Err(BittorrentError::BencodeError(
            "Unterminated list".to_string(),
        ));
    }

    *pos += 1; // Skip 'e'

    Ok(BencodeValue::List(list))
}

fn decode_dict(data: &[u8], pos: &mut usize) -> Result<BencodeValue> {
    *pos += 1; // Skip 'd'

    let mut dict = BTreeMap::new();

    while *pos < data.len() && data[*pos] != b'e' {
        // Keys must be strings
        let key = match decode_string(data, pos)? {
            BencodeValue::String(k) => k,
            _ => {
                return Err(BittorrentError::BencodeError(
                    "Dictionary key must be a string".to_string(),
                ))
            }
        };

        let value = decode_value(data, pos)?;
        dict.insert(key, value);
    }

    if *pos >= data.len() {
        return Err(BittorrentError::BencodeError(
            "Unterminated dictionary".to_string(),
        ));
    }

    *pos += 1; // Skip 'e'

    Ok(BencodeValue::Dict(dict))
}
