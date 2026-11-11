//! Exact admitted command DATA retained by a mandatory material use.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct PreparedCorrectionSnapshot {
    pub(in crate::domain_computation::primary_graph) key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) intent: [u8; 32],
    pub(in crate::domain_computation::primary_graph) action: String,
    pub(in crate::domain_computation::primary_graph) input: Vec<u8>,
    pub(in crate::domain_computation::primary_graph) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) actor: EntityId,
    pub(in crate::domain_computation::primary_graph) scope: RecordedScope,
    pub(in crate::domain_computation::primary_graph) target: RecordedCorrectionTarget,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation::primary_graph) struct RecordedCorrectionTarget {
    pub(in crate::domain_computation::primary_graph) owner:
        worth_foundational::facade::DurableWorldOwnerIdentity,
    pub(in crate::domain_computation::primary_graph) branch: String,
    pub(in crate::domain_computation::primary_graph) incarnation: u64,
    pub(in crate::domain_computation::primary_graph) generation: u64,
    pub(in crate::domain_computation::primary_graph) commit: u64,
    pub(in crate::domain_computation::primary_graph) relational: RelationalCommitReceipt,
    pub(in crate::domain_computation::primary_graph) program: [u8; 32],
}

impl RecordedCorrectionTarget {
    pub(in crate::domain_computation::primary_graph) fn matches(
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
