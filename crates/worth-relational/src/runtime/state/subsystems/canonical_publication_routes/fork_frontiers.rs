//! Observe both original streams under their existing position reservation.
use super::RelationalCanonicalPublicationRoutes;
use crate::branch::RelationalBranchTransitionSequence;
use crate::publication::patch::data::PatchStreamPosition;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn observe_fork_original_frontiers(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(PatchStreamPosition, RelationalBranchTransitionSequence), WorkCeilingDenial> {
        let (_reservation, canonical) = self.performed_stream.hold_reservation_within(execution)?;
        Ok((canonical, self.branch_transition_head()))
    }
}
