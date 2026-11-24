//! The recorded-original journey observes busy, stopped and terminal retirement
//! on one actual receiver. The retention lane still owns both gate arms.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::durability::authority::{
    RelationalRecoveredPublicationRetirement, RelationalRetentionGateTrialForTest,
};
use crate::history::retention::{
    RelationalExternalRetentionTerminalCounts, RelationalRetentionGateForTest,
};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    #[doc(hidden)]
    pub fn recovered_retirement_busy_trials_for_test(
        &self,
        task: &mut RelationalRecoveredPublicationRetirement,
        basis: &AdmittedRelationalBranchBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Option<[RelationalRetentionGateTrialForTest; 2]> {
        if basis.descriptor().runtime_instance_id() != self.runtime_instance_id() {
            return None;
        }
        let cut = task.advance_with_retention_gate_held_for_test(
            RelationalRetentionGateForTest::ExternalCut,
            basis,
            request,
        )?;
        let lane = task.advance_with_retention_gate_held_for_test(
            RelationalRetentionGateForTest::Lane,
            basis,
            request,
        )?;
        Some([cut, lane])
    }

    #[doc(hidden)]
    pub fn recovered_retirement_lane_releases_for_test(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Option<u64> {
        (basis.descriptor().runtime_instance_id() == self.runtime_instance_id()).then(|| {
            basis
                .inner
                .retention_binding
                .census_for_test(basis.inner.root.id())
                .counters
                .external_pin_releases
        })
    }

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
