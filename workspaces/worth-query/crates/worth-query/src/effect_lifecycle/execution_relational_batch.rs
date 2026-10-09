use worth_relational::facade::commit_strategies::{
    CanonicalStrategyCommitRequest, StrategyExecutionDraft,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{CommitResult, MutationIntent, WorkerIntentBatch};

use crate::workflow::LoweredMutationIntentDeclaration;

use super::execution::{lower_runtime_error, EffectExecutionDenialKind};
use super::execution_relational_scalar::{
    ensure_exact_basis_freshness, mutation_target_branch, observe_exact_branch_basis,
    open_exact_basis_snapshot,
};
use super::RelationalEffectExecutionFailure;

pub(super) fn execute_lowered_mutation_batch(
    runtime: &mut RelationalRuntime,
    declarations: &[LoweredMutationIntentDeclaration],
) -> Result<CommitResult, RelationalEffectExecutionFailure> {
    let first_declaration = declarations.first().ok_or_else(|| {
        (
            EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
            "batch-native mutation execution requires at least one declaration".to_string(),
        )
    })?;
    let target_branch = mutation_target_branch(first_declaration)?;
    for declaration in declarations.iter().skip(1) {
        let component_target = mutation_target_branch(declaration)?;
        if component_target != target_branch {
            return Err((
                EffectExecutionDenialKind::RelationalAuthorityBindingMalformed,
                format!(
                    "batch-native mutation execution cannot mix target branches `{}` and `{}`",
                    target_branch.0, component_target.0
                ),
            )
                .into());
        }
    }
    let transaction_validation_input = observe_exact_branch_basis(runtime, &target_branch)?;
    for declaration in declarations {
        ensure_exact_basis_freshness(declaration, &transaction_validation_input)?;
    }
    let snapshot =
        open_exact_basis_snapshot(runtime, &target_branch, &transaction_validation_input)?;
    let lowered_components = declarations
        .iter()
        .map(|declaration| {
            lower_batch_component(
                runtime,
                declaration,
                &transaction_validation_input,
                &snapshot,
            )
        })
        .collect::<Result<Vec<_>, _>>();
    super::exact_snapshot_closeout::release_exact_execution_snapshot(runtime, &snapshot);
    let lowered_components = lowered_components?;
    let mut batch = WorkerIntentBatch::new("worth-query-effect-batch");
    for intents in lowered_components {
        for intent in intents {
            batch.intents.push(intent);
        }
    }
    let mut txn = runtime
        .begin_branch_transaction(
            &transaction_validation_input,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .map_err(super::relational_execution_deferred::transaction_admission)?;
    txn.push_batch(
        batch,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    )
    .map_err(super::relational_execution_deferred::transaction_staging)?;
    let candidate = runtime
        .prepare_branch_transaction(
            txn,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(super::relational_execution_deferred::transaction_commit)?;
    let performed = match runtime.publication_port().compare_and_publish(candidate) {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => {
            performed
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Interrupted(event) => {
            return Err(super::relational_execution_deferred::interruption_event(
                event,
            ));
        }
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Deferred(deferred) => {
            return Err(super::relational_execution_deferred::publication(deferred));
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
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Failed(failure) => {
            return Err(super::relational_execution_deferred::publication_failure(
                failure,
            ));
        }
    };
    runtime
        .settle_performed_publication(performed)
        .map_err(|error| {
            let settlement = error.deferred_settlement().cloned();
            let (kind, message) =
                lower_runtime_error(error, EffectExecutionDenialKind::RelationalCommitFailed);
            RelationalEffectExecutionFailure::from_publication_failure(kind, message, settlement)
        })
}

fn lower_batch_component(
    runtime: &mut RelationalRuntime,
    declaration: &LoweredMutationIntentDeclaration,
    transaction_basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<Vec<MutationIntent>, (EffectExecutionDenialKind, String)> {
    let canonical: CanonicalStrategyCommitRequest = runtime
        .commit_strategies()
        .canonicalize_request(declaration.strategy_request())
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyCanonicalizationFailed,
            )
        })?;
    let execution: StrategyExecutionDraft = runtime
        .commit_strategies()
        .execute(&canonical, snapshot)
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyExecutionFailed,
            )
        })?;
    let mut authority = runtime.commit_strategies_authority();
    let lowered = authority
        .lower_execution(
            runtime,
            &canonical,
            &execution,
            transaction_basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .map_err(|error| {
            lower_runtime_error(
                error,
                EffectExecutionDenialKind::RelationalStrategyAuthorityLoweringFailed,
            )
        })?;
    Ok(lowered.merged_plan().merged_intents.clone())
}
