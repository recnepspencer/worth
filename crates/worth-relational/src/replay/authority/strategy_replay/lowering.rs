use super::{basis_admission, strategy_mismatch};
use crate::capabilities::RuntimeConfigSource;
use crate::commit_strategies::data::{
    StrategyCommitArtifactBundle, StrategyExecutionDraft, StrategyLoweringError,
};
use crate::durability::data::{RecoveryFailureClass, RelationalRecoveryStop};
use crate::replay::data::{CanonicalCommitEnvelope, ReplayMismatch, ReplayMismatchClass};
use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CommitPreparationReason, TransactionCommitError};

pub(super) fn lower_strategy_replay(
    basis_runtime: &RelationalRuntime,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
    expected_artifacts: &StrategyCommitArtifactBundle,
    execution: &StrategyExecutionDraft,
    mismatches: &mut Vec<ReplayMismatch>,
) -> Result<Option<StrategyCommitArtifactBundle>, RelationalRecoveryStop> {
    let replay_request = expected_artifacts.replay_request();
    let options = match basis_admission::replay_transaction_context(
        basis_runtime,
        &envelope.branch_context,
        &envelope.merge_parent_branches,
    ) {
        Ok(context) => context,
        Err(denial) => {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyLoweringDrift,
                format!("strategy replay transaction basis was denied: {denial:?}"),
                expected_artifacts,
                None,
            ));
            return Ok(None);
        }
    };
    let mut strategy_authority = basis_runtime.commit_strategies_authority();
    let lowered = match strategy_authority.lower_execution_with_input(
        request,
        basis_runtime,
        &replay_request,
        execution,
        options,
    ) {
        Ok(lowered) => lowered,
        Err(StrategyLoweringError::Preparation(error))
            if matches!(
                &error,
                TransactionCommitError::Execution { .. }
                    | TransactionCommitError::Interrupted { .. }
            ) || matches!(&error, TransactionCommitError::Preparation { error, .. }
                    if matches!(error.reason(), CommitPreparationReason::StorageUnavailable(_))) =>
        {
            return Err(crate::durability::authority::replay_commit_stop(
                error,
                "strategy replay preparation could not read committed storage".to_owned(),
            ));
        }
        Err(error) => {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyLoweringDrift,
                format!("strategy replay re-lowering failed: {error:?}"),
                expected_artifacts,
                None,
            ));
            return Ok(None);
        }
    };
    let descriptor = basis_runtime
        .commit_strategy_registry()
        .get_by_id(replay_request.strategy_id())
        .expect("replayed strategy request should resolve to a registered descriptor")
        .descriptor()
        .clone();
    if expected_artifacts.preview_validation_summary().is_some()
        || expected_artifacts.preview_validation_cost().is_some()
        || expected_artifacts.validated_against_version_id().is_some()
    {
        if let Err(detail) = basis_runtime
            .history
            .prepare_replay_target_sequence(envelope.commit.commit_id, envelope.commit.version_id)
        {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyLoweringDrift,
                detail.to_string(),
                expected_artifacts,
                None,
            ));
            return Ok(None);
        }
        match strategy_authority.validate_lowered_plan(request, basis_runtime, lowered) {
            Ok(validated) => Ok(validated.strategy_commit_artifacts().cloned()),
            Err(error) => {
                let detail = format!("strategy replay re-validation failed: {error:?}");
                let error = crate::execution::allocation_commit_error(error);
                match crate::durability::authority::replay_commit_stop(error, detail.clone()) {
                    RelationalRecoveryStop::Request(cause) => {
                        return Err(RelationalRecoveryStop::Request(cause))
                    }
                    RelationalRecoveryStop::Allocation(error) => {
                        return Err(RelationalRecoveryStop::Allocation(error))
                    }
                    RelationalRecoveryStop::Durability(error)
                        if error.class == RecoveryFailureClass::ColdStorageUnavailable =>
                    {
                        return Err(RelationalRecoveryStop::Durability(error));
                    }
                    RelationalRecoveryStop::Durability(_) => {}
                }
                mismatches.push(strategy_mismatch(
                    ReplayMismatchClass::StrategyLoweringDrift,
                    detail,
                    expected_artifacts,
                    None,
                ));
                Ok(None)
            }
        }
    } else {
        Ok(Some(StrategyCommitArtifactBundle::from_lowered(
            &lowered,
            &descriptor,
            basis_runtime.runtime_config(),
        )))
    }
}
