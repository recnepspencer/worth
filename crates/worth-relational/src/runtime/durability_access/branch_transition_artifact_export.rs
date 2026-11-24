use worth_relational_durability::internal_port::durability::branch_transition_artifact::retained_export::RelationalBranchTransitionArtifactsOwnerPort as _;
use super::DurabilityAccess;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifacts;
use worth_execution::ExecutionAllocationPolicy;

impl DurabilityAccess<'_> {
    /// The runtime's retained branch transitions, in increasing sequence, as
    /// artifacts. Durable captures release the covered prefix; memory-only
    /// runtimes keep it for older replay chains. This reads the runtime's own
    /// transition stream, not a second history store.
    ///
    /// The selection is fixed when this is called. Encoding is lazy: each
    /// item is framed only when pulled, under `policy`. Cost: O(performed
    /// tail transitions) shared handles up front, then one artifact encode per
    /// pulled item.
    pub fn branch_transition_artifacts<'scope, 'authority>(
        &self,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> RelationalBranchTransitionArtifacts<'scope, 'authority> {
        let (covered, transitions) = self.runtime.history.retained_branch_transitions();
        RelationalBranchTransitionArtifacts::new(covered, transitions, policy)
    }
}
