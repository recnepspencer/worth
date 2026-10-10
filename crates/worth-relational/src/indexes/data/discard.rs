use super::DerivedIndexId;

/// Remove an index's currently catalogued generations at every retained basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedIndexDiscardRequest {
    index_id: DerivedIndexId,
}

impl DerivedIndexDiscardRequest {
    pub const fn all_bases(index_id: DerivedIndexId) -> Self {
        Self { index_id }
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }
}

/// Catalog removal evidence, without retaining the removed generation payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedIndexDiscardOutcome {
    index_id: DerivedIndexId,
    removed_generation_count: usize,
}

impl DerivedIndexDiscardOutcome {
    pub(crate) const fn new(index_id: DerivedIndexId, removed_generation_count: usize) -> Self {
        Self {
            index_id,
            removed_generation_count,
        }
    }

    pub const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }

    pub const fn removed_generation_count(&self) -> usize {
        self.removed_generation_count
    }
}

/// Refusal to discard generations for an index absent from this installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexDiscardDenial {
    IndexNotInstalled { index_id: DerivedIndexId },
}
