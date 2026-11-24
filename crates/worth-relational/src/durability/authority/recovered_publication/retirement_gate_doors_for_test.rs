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
    /// Try the same receiver at each held gate, then advance normally once.
    /// The last count is the predecessor lane's explicit releases after that advance.
    #[doc(hidden)]
    pub fn advance_with_retention_gate_held_for_test(
        &mut self,
        basis: &AdmittedRelationalBranchBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Option<([RelationalRetentionGateTrialForTest; 2], Progress, u64)> {
        let mut trial = |gate| {
            basis
                .with_gate_held_for_test(gate, || self.advance(request))
                .map(
                    |(progress, before, after)| RelationalRetentionGateTrialForTest {
                        progress,
                        before,
                        after,
                    },
                )
        };
        let cut = trial(RelationalRetentionGateForTest::ExternalCut)?;
        let lane = trial(RelationalRetentionGateForTest::Lane)?;
        let progress = self.advance(request);
        let releases = basis
            .retention_census_for_test()
            .counters
            .external_pin_releases;
        Some(([cut, lane], progress, releases))
    }
}
