mod decoder;
mod encoder;
mod value;

pub use decoder::decode;
pub use encoder::encode;
pub use value::BencodeValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_encoding() {
        let value = BencodeValue::Integer(42);
        let encoded = encode(&value);
        assert_eq!(encoded, b"i42e");
    }

    #[test]
    fn test_string_encoding() {
        let value = BencodeValue::String(b"spam".to_vec());
        let encoded = encode(&value);
        assert_eq!(encoded, b"4:spam");
    }

    #[test]
    fn test_list_encoding() {
        let value = BencodeValue::List(vec![
            BencodeValue::String(b"spam".to_vec()),
            BencodeValue::Integer(42),
        ]);
        let encoded = encode(&value);
        assert_eq!(encoded, b"l4:spami42ee");
    }

    #[test]
    fn test_dict_encoding() {
        let mut dict = std::collections::BTreeMap::new();
        dict.insert(b"foo".to_vec(), BencodeValue::Integer(42));
        dict.insert(b"bar".to_vec(), BencodeValue::String(b"spam".to_vec()));
        let value = BencodeValue::Dict(dict);
        let encoded = encode(&value);
        assert_eq!(encoded, b"d3:bar4:spam3:fooi42ee");
    }

    #[test]
    fn test_roundtrip() {
        let original = BencodeValue::List(vec![
            BencodeValue::Integer(123),
            BencodeValue::String(b"test".to_vec()),
        ]);
        let encoded = encode(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
