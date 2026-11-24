use crate::branch::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::durability::data::RelationalRecoveryStop;

impl super::DurabilityAccess<'_> {
    /// Reconstruct transitions strictly after the selected basis. Read only
    /// the gap between that basis and the retained live tail from the medium.
    /// Cost: O(missing gap + retained tail), never O(covered history).
    pub(crate) fn reconstruct_branch_transition_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        after: RelationalBranchTransitionSequence,
    ) -> Result<Vec<PositionedBranchTransition>, RelationalRecoveryStop> {
        let (covered, tail) = self.runtime.history.retained_branch_transitions();
        let medium = if covered > after {
            self.runtime.durability.transition_history()
        } else {
            None
        };
        crate::durability::recovery_transition_history::reconstruct_transition_history(
            execution, after, covered, tail, medium,
        )
    }
}
