//! Test doors at the recovered retirement receiver and its basis lanes.
//!
//! One exact certification court restores the original busy-and-stop
//! retirement campaign through these doors. `advance_with_retention_gate_held_for_test`
//! holds one of the owner's existing retention gates (external cut or lane
//! lock) while this receiver advances once through its ordinary `advance`; it
//! adds no state machine and skips no gate. The census and terminal reads are
//! read-only. No production code reaches them; their caller row is in
//! `tools/boundary-check/config/control-callers.txt`.
use super::RelationalRecoveredPublicationRetirement;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::change_source::RelationalRetainedObservationRetirementProgress as Progress;
use crate::history::retention::{
    RelationalExternalRetentionTerminalCounts, RelationalRetentionCensusForTest,
    RelationalRetentionGateForTest,
};
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
    pub fn advance_with_retention_gate_held_for_test(
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

impl AdmittedRelationalBranchBasis {
    /// Read-only census of this basis's retention lane and its root.
    #[doc(hidden)]
    pub fn retention_census_for_test(&self) -> RelationalRetentionCensusForTest {
        self.inner
            .retention_binding
            .census_for_test(self.inner.root.id())
    }
}

impl crate::runtime::RelationalRuntime {
    /// Read-only count of explicit and dropped external retention releases.
    #[doc(hidden)]
    pub fn external_retention_terminal_counts_for_test(
        &self,
    ) -> RelationalExternalRetentionTerminalCounts {
        self.services
            .instrumentation
            .external_retention_terminal_accounting()
            .snapshot()
    }
}
