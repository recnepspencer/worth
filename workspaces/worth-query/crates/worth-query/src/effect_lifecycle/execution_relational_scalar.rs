use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::LoweredMutationIntentDeclaration;
use worth_relational::facade::commit_strategies::{
    CanonicalStrategyCommitRequest, StrategyExecutionDraft,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::transactions::CommitResult;

use super::execution::{lower_runtime_error, EffectExecutionDenialKind};
use super::RelationalEffectExecutionFailure;

pub(super) fn execute_lowered_mutation(
    runtime: &mut RelationalRuntime,
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    declaration: &LoweredMutationIntentDeclaration,
) -> Result<CommitResult, RelationalEffectExecutionFailure> {
    let target_branch = mutation_target_branch(declaration)?;
    let transaction_validation_input = observe_exact_branch_basis(runtime, &target_branch)?;
    ensure_exact_basis_freshness(declaration, &transaction_validation_input)?;
    let canonical: CanonicalStrategyCommitRequest = runtime
        .commit_strategies()
        .canonicalize_request(declaration.strategy_request())
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyCanonicalizationFailed,
            )
        })?;
    let snapshot =
        open_exact_basis_snapshot(runtime, &target_branch, &transaction_validation_input)?;
    let execution = runtime
        .commit_strategies()
        .execute(&canonical, &snapshot, request_execution)
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyExecutionFailed,
            )
        });
    super::exact_snapshot_closeout::release_exact_execution_snapshot(runtime, &snapshot);
    let execution: StrategyExecutionDraft = execution?;
    let mut commit_authority = runtime.commit_strategies_authority();
    let lowered = commit_authority
        .lower_execution(
            runtime,
            &canonical,
            &execution,
            &transaction_validation_input,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyAuthorityLoweringFailed,
            )
        })?;
    let validated = commit_authority
        .validate_lowered_plan(request_execution, runtime, lowered)
        .map_err(super::relational_execution_deferred::transaction_commit)?;
    let candidate = commit_authority
        .prepare_validated_commit(request_execution, runtime, validated)
        .map_err(super::relational_execution_deferred::transaction_commit)?;
    let performed = match runtime
        .publication_port()
        .compare_and_publish(request_execution, candidate)
    {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::RequestStopped(stop) => {
            return Err(super::relational_execution_deferred::execution_denial(
                stop.into(),
            ));
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => {
            performed
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Stale(stale) => {
            return Err(lower_runtime_error(
                stale,
                EffectExecutionDenialKind::RelationalCommitFailed,
            )
            .into());
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Denied(denial) => {
            return Err(lower_runtime_error(
                denial,
                EffectExecutionDenialKind::RelationalCommitFailed,
            )
            .into());
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Interrupted(event) => {
            return Err(super::relational_execution_deferred::interruption_event(
                event,
            ));
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Deferred(deferred) => {
            return Err(super::relational_execution_deferred::publication(deferred));
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Failed(failure) => {
            return Err(super::relational_execution_deferred::publication_failure(
                failure,
            ));
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::DurabilityUnresolved(
            unresolved,
        ) => {
            return Err(RelationalEffectExecutionFailure::durability_unresolved(
                unresolved,
            ));
        }
    };
    runtime
        .settle_performed_publication(performed)
        .map_err(|error| {
            lower_runtime_error(error, EffectExecutionDenialKind::RelationalCommitFailed).into()
        })
}

pub(crate) fn mutation_target_branch(
    declaration: &LoweredMutationIntentDeclaration,
) -> Result<BranchId, (EffectExecutionDenialKind, String)> {
    declaration
        .authority_binding()
        .runtime_target_branch()
        .cloned()
        .ok_or_else(|| {
            (
                EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
                format!(
                    "lowered relational mutation execution requires a typed target branch for authority binding `{}`",
                    declaration.authority_binding().binding_digest()
                ),
            )
        })
}

pub(super) fn ensure_exact_basis_freshness(
    declaration: &LoweredMutationIntentDeclaration,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<(), RelationalEffectExecutionFailure> {
    let Some(expected_snapshot_identity) =
        declaration.authority_binding().runtime_snapshot_identity()
    else {
        return Ok(());
    };
    let target_branch = mutation_target_branch(declaration)?;
    let observed_snapshot_identity = snapshot_identity_from_exact_basis(basis, &target_branch)?;
    let expected_snapshot_evidence = expected_snapshot_identity.evidence_identity();
    let observed_snapshot_evidence = observed_snapshot_identity.evidence_identity();
    if expected_snapshot_evidence == observed_snapshot_evidence {
        return Ok(());
    }
    Err(RelationalEffectExecutionFailure::Denied {
        kind: EffectExecutionDenialKind::RelationalExactBasisStale,
        message: format!(
            "lowered relational mutation execution preserved runtime snapshot `{}` for branch `{}` but current authority state is `{}`",
            expected_snapshot_evidence.reporting_projection(),
            target_branch.0
            ,
            observed_snapshot_evidence.reporting_projection()
        ),
    })
}

fn snapshot_identity_from_exact_basis(
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    branch: &BranchId,
) -> Result<WorthQuerySnapshotIdentity, RelationalEffectExecutionFailure> {
    crate::memory_workspace::snapshot_identity_from_admitted_basis(basis).ok_or_else(|| {
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
            message: format!(
                "lowered relational mutation execution found no current head for branch `{}`",
                branch.0
            ),
        }
    })
}

pub(super) fn open_exact_basis_snapshot(
    runtime: &mut RelationalRuntime,
    branch: &BranchId,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<SnapshotHandle, RelationalEffectExecutionFailure> {
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(|denial| {
            let kind = match denial {
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                } => return RelationalEffectExecutionFailure::deferred(
                    super::EffectExecutionDeferredKind::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    },
                    format!("owner rejected exact mutation snapshot for branch `{}`: {denial:?}", branch.0),
                ),
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
                    EffectExecutionDenialKind::RelationalAuthorityBindingMalformed
                }
                worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                    EffectExecutionDenialKind::SnapshotIdentityExhausted
                }
            };
            RelationalEffectExecutionFailure::Denied { kind, message: format!(
                "owner rejected exact mutation snapshot for branch `{}`: {denial:?}",
                branch.0,
            )}
        })
}

pub(super) fn observe_exact_branch_basis(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> Result<
    worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    RelationalEffectExecutionFailure,
> {
    let identity = runtime.branch_identity(branch).map_err(|denial| {
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
            message: format!("owner rejected mutation branch `{}`: {denial:?}", branch.0),
        }
    })?;
    runtime
        .observe_branch(&identity)
        .map(|(_, basis)| basis)
        .map_err(|denial| {
            match denial {
                worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
                    RelationalEffectExecutionFailure::deferred(
                        super::EffectExecutionDeferredKind::RetentionBackpressure,
                        format!(
                            "owner deferred current mutation basis for branch `{}`: {denial:?}",
                            branch.0
                        ),
                    )
                }
                worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
                    RelationalEffectExecutionFailure::Denied {
                        kind: EffectExecutionDenialKind::TransactionRetentionIdentityExhausted,
                        message: format!(
                            "owner rejected current mutation basis for branch `{}`: {denial:?}",
                            branch.0
                        ),
                    }
                }
                worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
                    RelationalEffectExecutionFailure::Denied {
                        kind: EffectExecutionDenialKind::SnapshotIdentityExhausted,
                        message: format!(
                            "owner rejected current mutation basis for branch `{}`: {denial:?}",
                            branch.0
                        ),
                    }
                }
                _ => RelationalEffectExecutionFailure::Denied {
                    kind: EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
                    message: format!(
                    "owner rejected current mutation basis for branch `{}`: {denial:?}",
                    branch.0
                ),
                },
            }
        })
}
