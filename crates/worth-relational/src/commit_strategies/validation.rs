use crate::capabilities::RuntimeConfigSource;
use crate::commit_strategies::data::{LoweredStrategyCommitPlan, StrategyCommitArtifactBundle};
use crate::mvcc::validation::StrategyProposalDecoration;
use crate::mvcc::ValidatedRelationalProposal;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::TransactionCommitError;
use crate::transactions::data::{CommitConflict, ConflictClass};

pub(crate) fn validate_lowered_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    lowered: LoweredStrategyCommitPlan,
) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
    lowered
        .transaction()
        .ensure_current_basis_for_runtime(runtime)
        .map_err(TransactionCommitError::conflict)?;
    let descriptor = runtime
        .commit_strategy_registry()
        .get_by_id(lowered.request().strategy_id())
        .ok_or_else(|| {
            TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::StaleValidationBasis {
                    detail: "lowered strategy descriptor is no longer registered".to_owned(),
                },
            ))
        })?
        .descriptor()
        .clone();
    let artifacts =
        StrategyCommitArtifactBundle::from_lowered(&lowered, &descriptor, runtime.runtime_config());
    let (transaction, bulk_mutation_batch, selected_branch_state, merged_plan) =
        lowered.into_validation_parts();

    runtime.validate_lowered_strategy_proposal(
        execution,
        transaction,
        selected_branch_state,
        merged_plan,
        StrategyProposalDecoration {
            artifacts,
            bulk_mutation_batch,
        },
    )
}
