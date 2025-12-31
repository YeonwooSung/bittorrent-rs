use super::BencodeValue;

/// Encode a BencodeValue into its byte representation
pub fn encode(value: &BencodeValue) -> Vec<u8> {
    let mut result = Vec::new();
    encode_into(value, &mut result);
    result
}

fn encode_into(value: &BencodeValue, output: &mut Vec<u8>) {
    match value {
        BencodeValue::Integer(i) => {
            output.push(b'i');
            output.extend_from_slice(i.to_string().as_bytes());
            output.push(b'e');
        }
        BencodeValue::String(s) => {
            output.extend_from_slice(s.len().to_string().as_bytes());
            output.push(b':');
            output.extend_from_slice(s);
        }
        BencodeValue::List(list) => {
            output.push(b'l');
            for item in list {
                encode_into(item, output);
            }
            output.push(b'e');
        }
        BencodeValue::Dict(dict) => {
            output.push(b'd');
            for (key, value) in dict {
                // Encode key as string
                output.extend_from_slice(key.len().to_string().as_bytes());
                output.push(b':');
                output.extend_from_slice(key);
                // Encode value
                encode_into(value, output);
            }
            output.push(b'e');
        }
    }
}
