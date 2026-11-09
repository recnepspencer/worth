use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

/// Canonical descriptive identity of one cross-runtime protocol family.
///
/// Version is deliberately separate: changing a produced version does not
/// rename the protocol, and consumers can declare an explicit compatibility
/// window over one stable family identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct BoundaryProtocolIdentity(Cow<'static, str>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryProtocolIdentityDenial {
    NonCanonical,
}

impl BoundaryProtocolIdentity {
    /// Declares a canonical dotted protocol family name.
    ///
    /// # Panics
    ///
    /// Panics when `value` is not a lowercase dotted identifier or ends in a
    /// version-shaped `vN` segment. Associated-constant use therefore rejects
    /// invalid declarations at compile time.
    ///
    /// ```compile_fail
    /// use worth_foundational::facade::BoundaryProtocolIdentity;
    ///
    /// const INVALID: BoundaryProtocolIdentity =
    ///     BoundaryProtocolIdentity::new("bank.estate.notice.v1");
    /// ```
    pub const fn new(value: &'static str) -> Self {
        if !is_canonical_protocol_identity(value) {
            panic!("boundary protocol identity must be canonical and version-free");
        }
        Self(Cow::Borrowed(value))
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, BoundaryProtocolIdentityDenial> {
        let value = value.into();
        Self::validate(&value)?;
        Ok(Self(Cow::Owned(value)))
    }

    /// Validate a borrowed family without allocating a descriptive owner.
    pub fn validate(value: &str) -> Result<(), BoundaryProtocolIdentityDenial> {
        if is_canonical_protocol_identity(value) {
            Ok(())
        } else {
            Err(BoundaryProtocolIdentityDenial::NonCanonical)
        }
    }

    /// Heap capacity retained by an owned parsed family; static declarations allocate nothing.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        match &self.0 {
            Cow::Borrowed(_) => 0,
            Cow::Owned(value) => value.capacity(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for BoundaryProtocolIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BoundaryProtocolIdentity {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(|BoundaryProtocolIdentityDenial::NonCanonical| {
            serde::de::Error::custom("invalid canonical boundary protocol identity")
        })
    }
}

const fn is_canonical_protocol_identity(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut segment_start = 0;
    let mut index = 0;
    let mut segments = 0;
    while index <= bytes.len() {
        if index == bytes.len() || bytes[index] == b'.' {
            if !is_canonical_segment(bytes, segment_start, index)
                || is_version_segment(bytes, segment_start, index)
            {
                return false;
            }
            segments += 1;
            segment_start = index + 1;
        }
        index += 1;
    }
    segments >= 2
}

const fn is_canonical_segment(bytes: &[u8], start: usize, end: usize) -> bool {
    if start == end || !is_lowercase_letter(bytes[start]) {
        return false;
    }
    let mut index = start + 1;
    let mut previous_hyphen = false;
    while index < end {
        let byte = bytes[index];
        if byte == b'-' {
            if previous_hyphen || index + 1 == end {
                return false;
            }
            previous_hyphen = true;
        } else if is_lowercase_letter(byte) || is_digit(byte) {
            previous_hyphen = false;
        } else {
            return false;
        }
        index += 1;
    }
    true
}

const fn is_version_segment(bytes: &[u8], start: usize, end: usize) -> bool {
    if end - start < 2 || bytes[start] != b'v' {
        return false;
    }
    let mut index = start + 1;
    while index < end {
        if !is_digit(bytes[index]) {
            return false;
        }
        index += 1;
    }
    true
}

const fn is_lowercase_letter(byte: u8) -> bool {
    byte >= b'a' && byte <= b'z'
}

const fn is_digit(byte: u8) -> bool {
    byte >= b'0' && byte <= b'9'
}

#[cfg(test)]
mod tests {
    use super::BoundaryProtocolIdentity;

    #[test]
    fn identity_is_version_free_and_round_trips_through_json() {
        let identity = BoundaryProtocolIdentity::new("bank.estate.death-notification");
        let encoded = serde_json::to_string(&identity).unwrap();
        let decoded: BoundaryProtocolIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn runtime_parser_rejects_versioned_or_noncanonical_names() {
        for invalid in [
            "bank.estate.notice.v1",
            "Bank.estate.notice",
            "bank..notice",
            "bank.notice-",
        ] {
            assert!(
                BoundaryProtocolIdentity::parse(invalid).is_err(),
                "{invalid}"
            );
        }
        for invalid_json in [
            "\"bank.estate.notice.v1\"",
            "\"Bank.estate.notice\"",
            "\"bank..notice\"",
        ] {
            assert!(serde_json::from_str::<BoundaryProtocolIdentity>(invalid_json).is_err());
        }
    }
}
