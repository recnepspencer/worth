//! Serialized record-history openings; the content owner verifies their hashes.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HistoryAppendFrontier {
    pub(in crate::storage) siblings: Vec<HistoryFrontierSibling>,
    pub(in crate::storage) last: Option<(u128, [u8; 32])>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HistoryFrontierSibling {
    pub(in crate::storage) key: u128,
    pub(in crate::storage) bit: u32,
    pub(in crate::storage) digest: [u8; 32],
    pub(in crate::storage) parent_bit: u32,
}

impl HistoryAppendFrontier {
    pub(crate) fn owned_allocation_bytes(&self) -> u64 {
        (self.siblings.capacity() * std::mem::size_of::<HistoryFrontierSibling>()) as u64
    }
}
