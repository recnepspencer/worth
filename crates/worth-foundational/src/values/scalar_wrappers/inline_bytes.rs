use serde::{de, Deserialize, Deserializer, Serialize};

/// Opaque inline payload with a carried proof that its native scalar body fits
/// the existing u32 frame. This proves width, not domain canonicality.
/// Ordering and hashing are representation mechanics, not domain comparisons.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct InlineBytes(Vec<u8>);

impl InlineBytes {
    /// One tag byte and four payload-length bytes share the u32 scalar frame.
    pub const MAXIMUM_LENGTH: usize = u32::MAX as usize - 5;

    /// O(1), moving the caller's allocation without copying or normalizing it.
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, InlineBytesLengthDenial> {
        Self::admit_length(bytes.len())?;
        Ok(Self(bytes))
    }

    /// Checks a wire length before a decoder allocates or copies the payload.
    pub const fn admit_length(length: usize) -> Result<(), InlineBytesLengthDenial> {
        if length > Self::MAXIMUM_LENGTH {
            Err(InlineBytesLengthDenial { observed: length })
        } else {
            Ok(())
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Exact width carried by construction and serde readmission.
    pub fn wire_length(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.0.capacity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineBytesLengthDenial {
    observed: usize,
}

impl InlineBytesLengthDenial {
    pub const fn observed(self) -> usize {
        self.observed
    }

    pub const fn maximum(self) -> usize {
        InlineBytes::MAXIMUM_LENGTH
    }
}

impl std::fmt::Display for InlineBytesLengthDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "inline payload length {} exceeds native framing maximum {}",
            self.observed,
            self.maximum()
        )
    }
}

impl std::error::Error for InlineBytesLengthDenial {}

impl<'de> Deserialize<'de> for InlineBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;
        impl<'de> de::Visitor<'de> for BytesVisitor {
            type Value = InlineBytes;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an inline byte sequence within the native framing maximum")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(
                self,
                mut input: A,
            ) -> Result<Self::Value, A::Error> {
                if let Some(length) = input.size_hint() {
                    InlineBytes::admit_length(length).map_err(de::Error::custom)?;
                }
                let mut bytes = Vec::new();
                while let Some(byte) = input.next_element::<u8>()? {
                    InlineBytes::admit_length(bytes.len() + 1).map_err(de::Error::custom)?;
                    bytes.try_reserve(1).map_err(de::Error::custom)?;
                    bytes.push(byte);
                }
                InlineBytes::try_new(bytes).map_err(de::Error::custom)
            }
        }
        deserializer.deserialize_seq(BytesVisitor)
    }
}

#[cfg(test)]
mod tests;
