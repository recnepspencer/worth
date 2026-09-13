use crate::logic::transaction::canonical_digest;
use crate::state::{SignalBranchId, SignalSnapshotId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignalBranchTransactionHead {
    branch_id: SignalBranchId,
    snapshot_id: Option<SignalSnapshotId>,
    generation: u64,
    head_digest: String,
}

impl SignalBranchTransactionHead {
    pub(super) fn new(
        branch_id: SignalBranchId,
        snapshot_id: Option<SignalSnapshotId>,
        generation: u64,
    ) -> Self {
        Self {
            branch_id,
            snapshot_id,
            generation,
            head_digest: canonical_digest(&(branch_id, snapshot_id, generation)),
        }
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    #[cfg(test)]
    pub fn snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.snapshot_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    #[cfg(test)]
    pub fn head_digest(&self) -> &str {
        &self.head_digest
    }
}
