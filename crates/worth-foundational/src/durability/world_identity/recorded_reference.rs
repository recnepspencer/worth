//! Recorded product heads preserve whether their owner ever claimed durability.
use super::{DurableProductBranchNameDenial, DurableProductBranchReference};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordedProductBranchReference {
    Home(DurableProductBranchReference),
    Memory(MemoryProductBranchReference),
}

/// A descriptive memory-frame address. It deliberately is not DurableIdentity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryProductBranchReference {
    owner: u64,
    branch: String,
    incarnation: u64,
    generation: u64,
    commit: u64,
}

impl MemoryProductBranchReference {
    pub fn from_observed(
        owner: u64,
        branch: String,
        incarnation: u64,
        generation: u64,
        commit: u64,
    ) -> Result<Self, DurableProductBranchNameDenial> {
        super::branch::validate_name(&branch)?;
        Ok(Self {
            owner,
            branch,
            incarnation,
            generation,
            commit,
        })
    }
    pub const fn owner(&self) -> u64 {
        self.owner
    }
    pub fn branch(&self) -> &str {
        &self.branch
    }
    pub const fn incarnation(&self) -> u64 {
        self.incarnation
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn selected_commit(&self) -> u64 {
        self.commit
    }
}

impl RecordedProductBranchReference {
    pub fn durable(&self) -> Option<&DurableProductBranchReference> {
        match self {
            Self::Home(value) => Some(value),
            Self::Memory(_) => None,
        }
    }
    pub fn memory(&self) -> Option<&MemoryProductBranchReference> {
        match self {
            Self::Home(_) => None,
            Self::Memory(value) => Some(value),
        }
    }
}
