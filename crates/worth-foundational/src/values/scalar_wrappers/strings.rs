use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Symbol(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "StoredStringWire")]
pub enum InternedString {
    Raw(String),
    Symbol(Symbol),
    #[serde(rename(serialize = "Raw"), skip_deserializing)]
    Stored(crate::value_shape::StoredValueText),
}

// A restored text artifact starts without any codec proof. Serialization keeps
// the existing Raw/Symbol wire vocabulary; seals are never durable authority.
#[derive(Deserialize)]
enum StoredStringWire {
    Raw(String),
    Symbol(Symbol),
}
impl From<StoredStringWire> for InternedString {
    fn from(value: StoredStringWire) -> Self {
        match value {
            StoredStringWire::Raw(text) => {
                Self::Stored(crate::value_shape::StoredValueText::new(text))
            }
            StoredStringWire::Symbol(symbol) => Self::Symbol(symbol),
        }
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for InternedString {}
impl PartialOrd for InternedString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for InternedString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Symbol(a), Self::Symbol(b)) => a.cmp(b),
            (Self::Symbol(_), _) => std::cmp::Ordering::Greater,
            (_, Self::Symbol(_)) => std::cmp::Ordering::Less,
            _ => self.raw_text().cmp(&other.raw_text()),
        }
    }
}
impl std::hash::Hash for InternedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Symbol(symbol) => {
                1_u8.hash(state);
                symbol.hash(state);
            }
            _ => {
                0_u8.hash(state);
                self.raw_text().hash(state);
            }
        }
    }
}

impl InternedString {
    pub fn into_raw_text(self) -> Option<String> {
        match self { Self::Raw(text) => Some(text), Self::Stored(text) => Some(text.as_str().to_owned()), Self::Symbol(_) => None }
    }

    pub fn raw_text(&self) -> Option<&str> {
        match self {
            Self::Raw(text) => Some(text),
            Self::Stored(text) => Some(text.as_str()),
            Self::Symbol(_) => None,
        }
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match self {
            Self::Raw(value) => value.capacity(),
            Self::Stored(value) => value.owned_allocation_capacity_bytes(),
            Self::Symbol(_) => 0,
        }
    }

    pub fn as_symbol(&self) -> Option<Symbol> {
        match self {
            Self::Raw(_) | Self::Stored(_) => None,
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
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let bytes = pairs
        .iter()
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
