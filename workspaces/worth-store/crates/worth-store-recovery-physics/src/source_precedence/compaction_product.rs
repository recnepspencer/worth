use worth_store_physical_integrity::VerifiedCheckpointCompactionCutover;

use super::PhysicalCheckpointBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedCompactionProduct {
    cutover: VerifiedCheckpointCompactionCutover,
}

impl SelectedCompactionProduct {
    pub fn admit(checkpoint: &PhysicalCheckpointBase) -> Self {
        let verified = checkpoint.checkpoint();
        let cutover = verified.compaction_cutover();
        Self { cutover }
    }

    pub const fn cutover(self) -> VerifiedCheckpointCompactionCutover {
        self.cutover
    }
}
