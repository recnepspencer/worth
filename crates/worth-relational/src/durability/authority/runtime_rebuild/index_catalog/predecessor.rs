//! Rebuild catalog-derived predecessor selections before ordinary tail application.
use super::RecoveredIndexCatalog;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::indexes::data::DerivedIndexMaintenanceDenialKind;
use crate::runtime::RelationalRuntime;

pub(in super::super) fn reconstruct_predecessor(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    restored: &mut RelationalRuntime,
    branch: &crate::history::data::BranchId,
    catalog: &RecoveredIndexCatalog,
) -> Result<(), RelationalRecoveryStop> {
    catalog
        .require_runtime(restored.runtime_instance_id())
        .map_err(|error| {
            super::super::envelope_replay::replay_commit_stop(
                error,
                "recovered index catalog belongs to another runtime".into(),
            )
        })?;
    if catalog.delta.definitions().is_empty() {
        return Ok(());
    }
    let basis = super::super::branch_readmission::recovered_branch_basis(restored, branch)?;
    let observation = basis.observation();
    let Some(_) = observation.commit_id() else {
        // The native pipeline already derives an authentic empty predecessor.
        return Ok(());
    };
    restored
        .index_authority()
        .reconstruct_installed_for_basis(execution, &basis)
        .map(|_| ())
        .map_err(|denial| match denial.kind {
            DerivedIndexMaintenanceDenialKind::Execution(cause) => {
                RelationalRecoveryStop::Request(cause)
            }
            DerivedIndexMaintenanceDenialKind::Basis(_)
            | DerivedIndexMaintenanceDenialKind::SnapshotUnavailable
            | DerivedIndexMaintenanceDenialKind::CommitMismatch
            | DerivedIndexMaintenanceDenialKind::BeforeRootMismatch
            | DerivedIndexMaintenanceDenialKind::IndexUnavailable(_)
            | DerivedIndexMaintenanceDenialKind::GenerationKindMismatch(_)
            | DerivedIndexMaintenanceDenialKind::PriorEntryMismatch
            | DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
            | DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
            | DerivedIndexMaintenanceDenialKind::ForeignCandidate
            | DerivedIndexMaintenanceDenialKind::CandidateLifetimeExpired { .. }
            | DerivedIndexMaintenanceDenialKind::CandidateUnavailable
            | DerivedIndexMaintenanceDenialKind::CandidateIndexesAlreadyPrepared
            | DerivedIndexMaintenanceDenialKind::GenerationIdentityExhausted => {
                DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    format!("recovered catalog predecessor reconstruction denied: {denial:?}"),
                )
                .into()
            }
        })
}
