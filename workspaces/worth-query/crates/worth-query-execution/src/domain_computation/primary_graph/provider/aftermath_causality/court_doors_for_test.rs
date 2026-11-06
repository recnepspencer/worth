//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn resolve_aftermath_causality_in_snapshot_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        pending: &WorthQueryPendingAftermathCausality,
        outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        Self::resolve_aftermath_causality_in_snapshot(
            self,
            execution,
            runtime,
            basis,
            snapshot,
            pending,
            outcome_identity,
        )
    }
}
