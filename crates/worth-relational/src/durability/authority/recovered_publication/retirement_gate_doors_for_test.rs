//! Exercise the original retirement receiver under an existing retention gate.
use super::RelationalRecoveredPublicationRetirement;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::retained_observation::retirement::RelationalRetainedObservationRetirementProgress as Progress;
use crate::history::retention::{RelationalRetentionCensusForTest, RelationalRetentionGateForTest};
use worth_execution::ExecutionRequest;

/// One gated advance: its ordinary progress and the lane census of the
/// witnessed basis before and while the gate was held.
#[doc(hidden)]
#[derive(Debug)]
pub struct RelationalRetentionGateTrialForTest {
    pub progress: Progress,
    pub before: RelationalRetentionCensusForTest,
    pub after: RelationalRetentionCensusForTest,
}

impl RelationalRecoveredPublicationRetirement {
    /// Advance once with `gate` of `basis`'s retention lane held. `None` when
    /// that lane's retention owner is gone; the receiver is then not advanced.
    #[doc(hidden)]
    pub(crate) fn advance_with_retention_gate_held_for_test(
        &mut self,
        gate: RelationalRetentionGateForTest,
        basis: &AdmittedRelationalBranchBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Option<RelationalRetentionGateTrialForTest> {
        basis
            .inner
            .retention_binding
            .with_gate_held_for_test(gate, basis.inner.root.id(), || self.advance(request))
            .map(
                |(progress, before, after)| RelationalRetentionGateTrialForTest {
                    progress,
                    before,
                    after,
                },
            )
    }
}
