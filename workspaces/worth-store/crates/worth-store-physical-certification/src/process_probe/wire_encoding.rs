use bincode::Options;
use serde::de::DeserializeOwned;
use serde::Serialize;

const MAX_WIRE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ENVIRONMENT_PAYLOAD_BYTES: usize = 12 * 1024;

pub(super) fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, String> {
    codec()
        .serialize(value)
        .map_err(|error| format!("could not encode process-probe evidence: {error}"))
}

pub(super) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    codec()
        .deserialize(bytes)
        .map_err(|error| format!("could not decode process-probe evidence: {error}"))
}

pub(super) fn encode_environment<T: Serialize + ?Sized>(value: &T) -> Result<String, String> {
    let bytes = encode(value)?;
    if bytes.len() > MAX_ENVIRONMENT_PAYLOAD_BYTES {
        return Err("process-probe environment payload exceeds its bound".to_owned());
    }
    Ok(hex::encode(&bytes))
}

pub(super) fn decode_environment<T: DeserializeOwned>(value: &str) -> Result<T, String> {
    if value.len() > MAX_ENVIRONMENT_PAYLOAD_BYTES * 2 {
        return Err("process-probe environment payload exceeds its bound".to_owned());
    }
    let bytes = hex::decode(value)?;
    if bytes.len() > MAX_ENVIRONMENT_PAYLOAD_BYTES {
        return Err("process-probe environment payload exceeds its bound".to_owned());
    }
    decode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_wire_codec_round_trips_without_accepting_trailing_bytes() {
        let encoded = encode(&(17_u64, "process-boundary".to_owned())).unwrap();
        assert_eq!(
            decode::<(u64, String)>(&encoded).unwrap(),
            (17, "process-boundary".to_owned())
        );

        let mut extended = encoded;
        extended.push(0);
        assert!(decode::<(u64, String)>(&extended).is_err());
    }

    #[test]
    fn oversized_environment_envelopes_are_rejected_before_hex_allocation() {
        let oversized = "00".repeat(MAX_ENVIRONMENT_PAYLOAD_BYTES + 1);
        assert_eq!(
            decode_environment::<Vec<u8>>(&oversized).unwrap_err(),
            "process-probe environment payload exceeds its bound"
        );
    }
}

fn codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(MAX_WIRE_BYTES)
        .reject_trailing_bytes()
}

mod hex {
    pub(super) fn encode(bytes: &[u8]) -> String {
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            use std::fmt::Write;
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
        }
        encoded
    }

    pub(super) fn decode(value: &str) -> Result<Vec<u8>, String> {
        if !value.len().is_multiple_of(2) {
            return Err("process-probe environment payload has invalid hex length".to_owned());
        }
        value
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let high = digit(pair[0])?;
                let low = digit(pair[1])?;
                Ok((high << 4) | low)
            })
            .collect()
    }

    fn digit(value: u8) -> Result<u8, String> {
        match value {
            b'0'..=b'9' => Ok(value - b'0'),
            b'a'..=b'f' => Ok(value - b'a' + 10),
            _ => Err("process-probe environment payload contains invalid hex".to_owned()),
        }
    }
}
