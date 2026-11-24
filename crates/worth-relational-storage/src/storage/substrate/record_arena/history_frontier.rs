//! Serialized record-history openings; the content owner verifies their hashes.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryAppendFrontier {
    pub siblings: Vec<HistoryFrontierSibling>,
    pub last: Option<(u128, worth_foundational::CanonicalDigestId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryFrontierSibling {
    pub(in crate::internal_port::storage) key: u128,
    pub(in crate::internal_port::storage) bit: u32,
    pub(in crate::internal_port::storage) digest: worth_foundational::CanonicalDigestId,
    pub(in crate::internal_port::storage) parent_bit: u32,
}

impl HistoryAppendFrontier {
    pub fn owned_allocation_bytes(&self) -> u64 {
        (self.siblings.capacity() * std::mem::size_of::<HistoryFrontierSibling>()) as u64
    }
}
