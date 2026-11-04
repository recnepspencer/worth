//! A digest's binary wire width is independent of its byte values.
use super::CanonicalDigestId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl Serialize for CanonicalDigestId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            let mut text = [0_u8; 64];
            for (index, byte) in self.bytes().iter().copied().enumerate() {
                text[index * 2] = HEX[usize::from(byte >> 4)];
                text[index * 2 + 1] = HEX[usize::from(byte & 15)];
            }
            let text = std::str::from_utf8(&text).expect("hexadecimal digits are UTF-8");
            serializer.serialize_str(text)
        } else {
            serializer.serialize_bytes(self.bytes())
        }
    }
}

impl<'de> Deserialize<'de> for CanonicalDigestId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(HumanDigest)
        } else {
            deserializer.deserialize_bytes(BinaryDigest)
        }
    }
}

struct BinaryDigest;
impl de::Visitor<'_> for BinaryDigest {
    type Value = CanonicalDigestId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 32-byte binary digest")
    }

    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        let bytes = bytes
            .try_into()
            .map_err(|_| E::invalid_length(bytes.len(), &self))?;
        Ok(CanonicalDigestId::new(bytes))
    }
}

struct HumanDigest;
impl<'de> de::Visitor<'de> for HumanDigest {
    type Value = CanonicalDigestId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("exactly 64 hexadecimal digest digits")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Self::Value, E> {
        CanonicalDigestId::parse_hex(text)
            .ok_or_else(|| E::invalid_value(de::Unexpected::Str(text), &self))
    }
}

#[cfg(test)]
mod tests;
