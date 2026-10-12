//! Consuming execution of one prepared merge.

use crate::authority::commit::pipeline::{
    execute_authoritative_commit, AuthoritativeCommitContext,
};
use crate::merge::data::{MergeExecutionError, PreparedMergeExecution};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{
    CommitConflict, ConflictClass, MergeExecutionOutcome, TransactionCommitError,
};

pub(in crate::merge) fn execute_prepared_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    prepared: PreparedMergeExecution,
) -> Result<MergeExecutionOutcome, MergeExecutionError> {
    let complexity_baseline = current_complexity_counters(runtime);
    runtime.performance_access().count_merge_execution_attempt();
    runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
        .map_err(|error| emit_failure(runtime, &prepared, error))?;
    let transaction_id = runtime.services.next_transaction_id();
    let mutation_plan = prepared.mutation_plan().bind_transaction(transaction_id);
    let target_identity = runtime
        .branch_identity(&mutation_plan.target_branch)
        .map_err(|denial| {
            MergeExecutionError::Commit(TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::InvalidMergeParent {
                    detail: format!(
                        "prepared merge target cannot issue an owner branch binding: {denial:?}"
                    ),
                },
            )))
        })?;
    let mut options = runtime
        .transaction_validation_input_for(&target_identity)
        .map_err(|denial| {
            MergeExecutionError::Commit(TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::InvalidMergeParent {
                    detail: format!(
                        "prepared merge target binding was denied by the owner: {denial:?}"
                    ),
                },
            )))
        })?;
    let parent_bindings = mutation_plan
        .merge_parent_branches
        .iter()
        .map(|branch| {
            let identity = runtime.branch_identity(branch).map_err(|denial| {
                MergeExecutionError::Commit(TransactionCommitError::conflict(CommitConflict::new(
                    ConflictClass::InvalidMergeParent {
                        detail: format!(
                            "prepared merge parent cannot issue an owner identity: {denial:?}"
                        ),
                    },
                )))
            })?;
            runtime
                .transaction_validation_input_for(&identity)
                .map(|options| options.basis().clone())
                .map_err(|denial| {
                    MergeExecutionError::Commit(TransactionCommitError::conflict(
                        CommitConflict::new(ConflictClass::InvalidMergeParent {
                            detail: format!(
                                "prepared merge parent binding was denied by the owner: {denial:?}"
                            ),
                        }),
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    options = options.with_merge_parent_bases(parent_bindings);
    let execution_summary = mutation_plan.merge_execution_summary.clone();
    let structural_summary = mutation_plan.structural_summary.clone();
    let diagnostics_plan = prepared.bound_executable_plan().diagnostics_plan.clone();
    let context = AuthoritativeCommitContext::from_prepared_merge(
        options,
        mutation_plan,
        diagnostics_plan,
        complexity_baseline,
    )
    .map_err(MergeExecutionError::from)
    .map_err(|error| emit_failure(runtime, &prepared, error))?;
    let commit = execute_authoritative_commit(execution, runtime, context)
        .map_err(MergeExecutionError::from)
        .map_err(|error| emit_failure(runtime, &prepared, error))?;
    Ok(MergeExecutionOutcome {
        commit,
        execution_summary,
        structural_summary,
    })
}

fn current_complexity_counters(
    runtime: &RelationalRuntime,
) -> crate::performance::data::RuntimeComplexityCounters {
    runtime
        .services
        .instrumentation
        .complexity_counters
        .lock()
        .expect("complexity counter lock poisoned")
        .clone()
}

fn emit_failure(
    runtime: &RelationalRuntime,
    prepared: &PreparedMergeExecution,
    error: MergeExecutionError,
) -> MergeExecutionError {
    super::super::execution_diagnostics::emit_merge_execution_failure_artifact(
        runtime, prepared, &error,
    );
    error
}
