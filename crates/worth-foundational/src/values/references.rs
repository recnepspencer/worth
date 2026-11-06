use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Descriptive identity of a complete immutable Native content frame.
/// Availability, format admission, and access require the Native field owner.
pub struct ContentRefId(pub [u8; 32]);

impl ContentRefId {
    /// Parse the single lowercase hexadecimal spelling; no admission is granted.
    pub fn from_hex(text: &str) -> Option<Self> {
        if text.len() != 64 {
            return None;
        }
        let mut digest = [0; 32];
        for (slot, pair) in digest.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
            let digit = |byte| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            };
            *slot = digit(pair[0])? * 16 + digit(pair[1])?;
        }
        Some(Self(digest))
    }
}

impl std::fmt::Display for ContentRefId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub u32);

impl PartitionId {
    pub const fn main() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocalSlot(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Generation(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId {
    pub partition_id: PartitionId,
    pub local_slot: LocalSlot,
    pub generation: Generation,
}

impl EntityId {
    pub const fn new(partition_id: PartitionId, local_slot: u64, generation: u32) -> Self {
        Self {
            partition_id,
            local_slot: LocalSlot(local_slot),
            generation: Generation(generation),
        }
    }
}
