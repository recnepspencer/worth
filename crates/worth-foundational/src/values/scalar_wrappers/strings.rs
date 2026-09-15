use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Symbol(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InternedString {
    Raw(String),
    Symbol(Symbol),
}

impl InternedString {
    pub fn as_symbol(&self) -> Option<Symbol> {
        match self {
            Self::Raw(_) => None,
            Self::Symbol(symbol) => Some(*symbol),
        }
    }
}

impl From<&str> for InternedString {
    fn from(value: &str) -> Self {
        Self::Raw(value.to_string())
    }
}

impl From<String> for InternedString {
    fn from(value: String) -> Self {
        Self::Raw(value)
    }
}

pub type CanonicalString = InternedString;

pub fn encode_canonical_text_hex(value: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.bytes() {
        encoded.push(DIGITS[usize::from(byte >> 4)] as char);
        encoded.push(DIGITS[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

pub fn decode_canonical_text_hex(value: &str) -> Option<String> {
    let pairs = value.as_bytes().chunks_exact(2);
    if !pairs.remainder().is_empty() {
        return None;
    }
    let bytes = pairs
        .map(|pair| {
            hex_nibble(pair[0])?
                .checked_mul(16)?
                .checked_add(hex_nibble(pair[1])?)
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_canonical_text_hex, encode_canonical_text_hex};

    #[test]
    fn canonical_text_hex_round_trips_unicode_and_rejects_foreign_spellings() {
        let value = "β;line\nbreak";
        let encoded = encode_canonical_text_hex(value);
        assert_eq!(decode_canonical_text_hex(&encoded).as_deref(), Some(value));
        assert!(decode_canonical_text_hex("A0").is_none());
        assert!(decode_canonical_text_hex("aβb").is_none());
        assert!(decode_canonical_text_hex("f").is_none());
        assert!(decode_canonical_text_hex("ff").is_none());
    }
}
