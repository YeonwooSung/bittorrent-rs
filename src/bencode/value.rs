use std::collections::BTreeMap;

/// Represents a bencoded value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodeValue {
    /// Integer: i<number>e
    Integer(i64),
    /// Byte string: <length>:<contents>
    String(Vec<u8>),
    /// List: l<values>e
    List(Vec<BencodeValue>),
    /// Dictionary: d<key-value pairs>e (keys are sorted)
    Dict(BTreeMap<Vec<u8>, BencodeValue>),
}

impl BencodeValue {
    /// Try to get this value as an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BencodeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get this value as a byte string
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            BencodeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as a UTF-8 string
    pub fn as_str(&self) -> Option<&str> {
        self.as_bytes()
            .and_then(|b| std::str::from_utf8(b).ok())
    }

    /// Try to get this value as a list
    pub fn as_list(&self) -> Option<&[BencodeValue]> {
        match self {
            BencodeValue::List(l) => Some(l),
            _ => None,
        }
    }

    /// Try to get this value as a dictionary
    pub fn as_dict(&self) -> Option<&BTreeMap<Vec<u8>, BencodeValue>> {
        match self {
            BencodeValue::Dict(d) => Some(d),
            _ => None,
        }
    }

    /// Get a value from a dictionary by key
    pub fn dict_get(&self, key: &[u8]) -> Option<&BencodeValue> {
        self.as_dict()?.get(key)
    }

    /// Get a string value from a dictionary by key
    pub fn dict_get_str(&self, key: &[u8]) -> Option<&str> {
        self.dict_get(key)?.as_str()
    }

    /// Get an integer value from a dictionary by key
    pub fn dict_get_int(&self, key: &[u8]) -> Option<i64> {
        self.dict_get(key)?.as_integer()
    }
}
