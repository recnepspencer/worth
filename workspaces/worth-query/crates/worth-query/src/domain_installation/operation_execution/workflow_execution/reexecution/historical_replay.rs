use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryBoundDomainOperation;
use crate::ordinary::history::WorthQueryHistoricalContext;
use crate::runtime::WorthQueryWorkspace;
use worth_proof::TransitionOutcome;

use super::certification_replay::{
    execute_admitted_replay, WorthQueryCertificationReplayAdmissionDenial,
    WorthQueryCertificationReplayCapability, WorthQueryCertificationReplayCounters,
    WorthQueryCertificationReplayOutcome, WorthQueryHistoricalBasisCorrespondence,
    WorthQueryHistoricalReplayAdmission, WorthQueryHistoricalReplayAdmissionDenial,
    WorthQueryReplayBasisRelationship,
};
use super::{
    WorthQueryCompletedWorkflowTrace, WorthQueryExecutableDomainOperation,
    WorthQueryNormalizedWorkflowIntent, WorthQueryWorkflowOperation,
};

/// How historical replay should reach the original run's basis, requested
/// from `admit_installed_historical_replay_basis`.
///
/// Only `RetainedSnapshot` is currently admitted; the other paths are denied
/// with `HistoricalExecutionSubstrateUnavailable`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledHistoricalReplayPath {
    /// Replay on the retained snapshot of the original run.
    RetainedSnapshot,
    /// Delta replay, with its event limit and the actual event count.
    DeltaReplay {
        max_events: usize,
        actual_events: usize,
    },
    /// Full reconstruction, with its scope limit and the actual scope.
    FullReconstruction {
        max_scope: usize,
        actual_scope: usize,
    },
}

/// Readmits Query's sealed retained-snapshot context for one exact installed
/// operation pair. The in-memory owner has no delta-replay or reconstruction
/// substrate, so those requests remain explicit typed denials.
pub fn admit_installed_historical_replay_basis<
    D,
    O,
    F,
    LO: BasisOperationLane,
    LR: BasisOperationLane,
>(
    _capability: WorthQueryCertificationReplayCapability,
    original: &WorthQueryCompletedWorkflowTrace<D, O, F, LO>,
    bound: &WorthQueryBoundDomainOperation<D, O, F, LR>,
    historical_context: &WorthQueryHistoricalContext,
    path: WorthQueryInstalledHistoricalReplayPath,
) -> TransitionOutcome<WorthQueryHistoricalReplayAdmission, WorthQueryHistoricalReplayAdmissionDenial>
{
    if !matches!(
        path,
        WorthQueryInstalledHistoricalReplayPath::RetainedSnapshot
    ) {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::HistoricalExecutionSubstrateUnavailable,
        );
    }
    let original_bound = original.bound();
    if original_bound.definition().canonical_identity() != bound.definition().canonical_identity() {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::ForeignOperation,
        );
    }
    if original_bound
        .operation()
        .domain_authority()
        .runtime_authority()
        != bound.operation().domain_authority().runtime_authority()
    {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::ForeignRuntime,
        );
    }
    if original_bound.operation().installation_generation()
        != bound.operation().installation_generation()
    {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::StaleInstallationGeneration,
        );
    }
    if original_bound.basis().capability_digest() != bound.basis().capability_digest() {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::ReplayBasisCapabilityMismatch,
        );
    }
    if original.stage_receipts().is_empty()
        || original
            .stage_receipts()
            .iter()
            .any(|receipt| !historical_context.admits_snapshot(receipt.execution_snapshot()))
    {
        return TransitionOutcome::Denied(
            WorthQueryHistoricalReplayAdmissionDenial::HistoricalSnapshotDoesNotBindOriginalTrace,
        );
    }

    TransitionOutcome::Success(WorthQueryHistoricalReplayAdmission {
        original_operation_identity: original_bound.definition().canonical_identity().to_owned(),
        replay_operation_identity: bound.definition().canonical_identity().to_owned(),
        original_basis_identity: original_bound.basis().capability_digest().to_owned(),
        execution_basis_identity: bound.basis().capability_digest().to_owned(),
        historical_workspace_name: historical_context.workspace_name().to_owned(),
        historical_snapshot_identity: historical_context.snapshot_identity().clone(),
        correspondence: WorthQueryHistoricalBasisCorrespondence::RetainedSnapshotIdentity,
    })
}

/// Re-executes a completed workflow trace on an admitted historical basis
/// and compares the replay with the original.
///
/// The admission must have been issued for this original trace and bound
/// operation, and the workspace must be the historical snapshot it names;
/// otherwise the replay is denied before anything runs. The remaining checks
/// and the comparison are those of `replay_installed_workflow`.
pub fn replay_installed_workflow_historical<
    D: 'static,
    O,
    F: 'static,
    LO: BasisOperationLane,
    LR: BasisOperationLane,
>(
    admission: WorthQueryHistoricalReplayAdmission,
    original: &WorthQueryCompletedWorkflowTrace<D, O, F, LO>,
    bound: WorthQueryBoundDomainOperation<D, O, F, LR>,
    intent: WorthQueryNormalizedWorkflowIntent,
    resources: worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryCertificationReplayOutcome<D, O, F, LR>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation> + 'static,
{
    let original_bound = original.bound();
    if admission.original_operation_identity != original_bound.definition().canonical_identity()
        || admission.replay_operation_identity != bound.definition().canonical_identity()
        || admission.original_basis_identity != original_bound.basis().capability_digest()
        || admission.execution_basis_identity != bound.basis().capability_digest()
    {
        return TransitionOutcome::Denied(
            super::certification_replay::WorthQueryCertificationReplayStop::Admission(
                WorthQueryCertificationReplayAdmissionDenial::HistoricalAdmissionMismatch,
            ),
        );
    }
    let snapshot = match workspace.snapshot_identity() {
        Ok(snapshot) => snapshot,
        Err(denial) => {
            return TransitionOutcome::Denied(
                super::certification_replay::WorthQueryCertificationReplayStop::Admission(
                    WorthQueryCertificationReplayAdmissionDenial::Handle(denial),
                ),
            )
        }
    };
    if admission.historical_workspace_name != workspace.name()
        || !admission
            .historical_snapshot_identity
            .is_same_current_identity_as(&snapshot)
    {
        return TransitionOutcome::Denied(
            super::certification_replay::WorthQueryCertificationReplayStop::Admission(
                WorthQueryCertificationReplayAdmissionDenial::HistoricalExecutionBasisDrift,
            ),
        );
    }
    execute_admitted_replay(
        original,
        bound,
        intent,
        resources,
        workspace,
        WorthQueryReplayBasisRelationship::AdmittedHistoricalBasis {
            correspondence: admission.correspondence,
        },
        WorthQueryCertificationReplayCounters {
            authority_checks: 1,
            operation_checks: 1,
            basis_checks: 1,
            ..Default::default()
        },
    )
}
