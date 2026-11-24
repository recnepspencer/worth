use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchDetail {
    DenseBitset(Vec<u64>),
}

impl PatchDetail {
    pub fn canonicalized(&self) -> Self {
        match self {
            Self::DenseBitset(bits) => Self::DenseBitset(bits.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchFragmentBudget {
    pub worker_local_fragments: bool,
    pub deterministic_merge_required: bool,
}

impl Default for PatchFragmentBudget {
    fn default() -> Self {
        Self {
            worker_local_fragments: true,
            deterministic_merge_required: true,
        }
    }
}

/// Operations used by the Relational owners.
pub trait PatchDetailOwnerPort: Sized {
    fn owned_allocation_capacity_bytes(&self) -> u64;
}

impl PatchDetailOwnerPort for PatchDetail {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::DenseBitset(bits) => (bits.capacity() * std::mem::size_of::<u64>()) as u64,
        }
    }
}
