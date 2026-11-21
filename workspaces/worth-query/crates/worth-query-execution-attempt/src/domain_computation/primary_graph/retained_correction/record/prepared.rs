//! Exact admitted command DATA retained by a mandatory material use.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedCorrectionSnapshot {
    pub key: [u8; 32],
    pub intent: [u8; 32],
    pub action: String,
    pub input: Vec<u8>,
    pub input_identity: [u8; 32],
    pub actor: EntityId,
    pub scope: RecordedScope,
    pub target: RecordedCorrectionTarget,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedCorrectionTarget {
    pub owner:
        worth_foundational::facade::DurableWorldOwnerIdentity,
    pub branch: String,
    pub incarnation: u64,
    pub generation: u64,
    pub commit: u64,
    pub relational: RelationalCommitReceipt,
    pub program: [u8; 32],
}

impl RecordedCorrectionTarget {
    pub fn matches(
        &self,
        target: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        target.owner_identity().durable() == Some(self.owner)
            && target.branch_identity().name().as_str() == self.branch
            && target.lifecycle_incarnation().ordinal() == self.incarnation
            && target.reference_generation().get() == self.generation
            && target.selected_commit().ordinal() == self.commit
            && target
                .basis()
                .relational_basis()
                .observation()
                .commit_receipt()
                == Some(&self.relational)
    }
}
