//! Observe both original streams under their existing position reservation.
use super::RelationalCanonicalPublicationRoutes;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational_contracts::internal_port::branch::transition::sequence::RelationalBranchTransitionSequence;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn observe_fork_original_frontiers(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(PatchStreamPosition, RelationalBranchTransitionSequence), WorkCeilingDenial> {
        let (_reservation, canonical) = self.performed_stream.hold_reservation_within(execution)?;
        Ok((canonical, self.branch_transition_head()))
    }
}
