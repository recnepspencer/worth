use crate::capabilities::RuntimeConfigSource;
use crate::commit_strategies::data::StrategyCommitArtifactBundle;
use crate::commit_strategies::data::StrategyExecutionDraft;
use crate::history::data::{BranchId, CommitId};
use crate::replay::data::{
    CanonicalCommitEnvelope, ReplayMismatch, ReplayMismatchClass, ReplayVerificationLayer,
    ReplayVerificationMode,
};
use crate::runtime::RelationalRuntime;

use super::super::planning::replay_commit_closure_by_commit_id_order;

mod basis_admission;
mod mismatch;

use mismatch::strategy_mismatch;

pub(super) fn verify_strategy_reexecution_surface(
    runtime: &RelationalRuntime,
    mismatches: &mut Vec<ReplayMismatch>,
    envelope: &CanonicalCommitEnvelope,
    commit_closure: &[CommitId],
    verification_mode: ReplayVerificationMode,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let Some(expected_artifacts) = envelope.strategy_artifacts.as_ref() else {
        return;
    };
    let Some(basis_commits) = select_strategy_basis_commits(
        runtime,
        envelope,
        expected_artifacts,
        commit_closure,
        mismatches,
    ) else {
        return;
    };
    let Some(basis_runtime) = rebuild_strategy_basis_runtime(
        request,
        runtime,
        envelope,
        expected_artifacts,
        &basis_commits,
        verification_mode,
        mismatches,
    ) else {
        return;
    };
    let Some(execution) =
        execute_strategy_replay(&basis_runtime, expected_artifacts, mismatches, request)
    else {
        return;
    };
    let Some(observed) = lower_strategy_replay(
        &basis_runtime,
        envelope,
        expected_artifacts,
        &execution,
        mismatches,
    ) else {
        return;
    };
    if strategy_artifacts_match(&execution, &observed, expected_artifacts) {
        runtime
            .performance_access()
            .count_replay_verification_layer(ReplayVerificationLayer::DeepArtifactParity);
        return;
    }
    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::DeepArtifactParity);
    mismatches.push(strategy_mismatch(
        ReplayMismatchClass::StrategyLoweringDrift,
        "strategy replay re-execution or re-lowering diverged from the committed strategy proof"
            .to_string(),
        expected_artifacts,
        Some(format!("{:?}", observed.replay_descriptor())),
    ));
}

fn strategy_artifacts_match(
    execution: &StrategyExecutionDraft,
    observed: &StrategyCommitArtifactBundle,
    expected_artifacts: &StrategyCommitArtifactBundle,
) -> bool {
    observed.lowering_provenance() == expected_artifacts.lowering_provenance()
        && observed.lowering_summary() == expected_artifacts.lowering_summary()
        && observed.preview_validation_summary() == expected_artifacts.preview_validation_summary()
        && observed.preview_validation_cost() == expected_artifacts.preview_validation_cost()
        && observed.validated_against_commit_id()
            == expected_artifacts.validated_against_commit_id()
        && observed.validated_against_version_id()
            == expected_artifacts.validated_against_version_id()
        && execution.output().digest() == expected_artifacts.replay_descriptor().output_digest()
        && execution.mutation_program().digest()
            == expected_artifacts
                .replay_descriptor()
                .mutation_program_digest()
        && execution.summary().projected_entity_record_reads
            == expected_artifacts
                .lowering_summary()
                .projected_entity_record_reads()
        && execution.summary().projected_relation_record_reads
            == expected_artifacts
                .lowering_summary()
                .projected_relation_record_reads()
        && execution.summary().projected_partition_reads
            == expected_artifacts
                .lowering_summary()
                .projected_partition_reads()
}

fn select_strategy_basis_commits(
    runtime: &RelationalRuntime,
    _envelope: &CanonicalCommitEnvelope,
    expected_artifacts: &StrategyCommitArtifactBundle,
    commit_closure: &[CommitId],
    mismatches: &mut Vec<ReplayMismatch>,
) -> Option<Vec<CommitId>> {
    let basis_commit = match (
        expected_artifacts.validated_against_commit_id(),
        expected_artifacts.validated_against_version_id(),
    ) {
        (Some(commit_id), _) => Some(commit_id),
        (None, Some(version_id)) if version_id.0 > 0 => {
            match runtime.history().commit_envelope_for_version(version_id) {
                Some(envelope) => Some(envelope.commit.commit_id),
                None => {
                    mismatches.push(strategy_mismatch(
                        ReplayMismatchClass::StrategyExecutorUnavailable,
                        format!(
                            "failed to resolve strategy replay validation basis version {version_id:?}"
                        ),
                        expected_artifacts,
                        None,
                    ));
                    return None;
                }
            }
        }
        _ => return Some(commit_closure[..commit_closure.len().saturating_sub(1)].to_vec()),
    };
    match replay_commit_closure_by_commit_id_order(runtime, runtime, basis_commit?) {
        Ok(chain) => Some(chain),
        Err(failure) => {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyExecutorUnavailable,
                format!("failed to rebuild strategy replay validation basis closure: {failure:?}"),
                expected_artifacts,
                None,
            ));
            None
        }
    }
}

fn rebuild_strategy_basis_runtime(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    expected_artifacts: &StrategyCommitArtifactBundle,
    basis_commits: &[CommitId],
    verification_mode: ReplayVerificationMode,
    mismatches: &mut Vec<ReplayMismatch>,
) -> Option<RelationalRuntime> {
    let basis_plan = super::super::planning::replay_recovery_plan_for_chain(
        runtime,
        basis_commits,
        verification_mode,
    );
    let basis_runtime = match runtime.rebuild_runtime_from_plan(execution, basis_plan) {
        Ok(runtime) => runtime,
        Err(error) => {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyExecutorUnavailable,
                format!(
                    "failed to rebuild pre-commit strategy replay basis: {}",
                    error.detail
                ),
                expected_artifacts,
                None,
            ));
            return None;
        }
    };
    if let Err(detail) =
        ensure_strategy_replay_basis_branch(&basis_runtime, envelope, expected_artifacts)
    {
        mismatches.push(strategy_mismatch(
            ReplayMismatchClass::StrategyExecutorUnavailable,
            detail,
            expected_artifacts,
            None,
        ));
        return None;
    }
    Some(basis_runtime)
}

fn execute_strategy_replay(
    basis_runtime: &RelationalRuntime,
    expected_artifacts: &StrategyCommitArtifactBundle,
    mismatches: &mut Vec<ReplayMismatch>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Option<StrategyExecutionDraft> {
    let replay_request = expected_artifacts.replay_request();
    let snapshot = basis_runtime.visibility_authority().snapshot();
    match basis_runtime
        .commit_strategies()
        .execute(&replay_request, &snapshot, request)
    {
        Ok(execution) => Some(execution),
        Err(error) => {
            let mismatch_class = match error {
                crate::commit_strategies::StrategyExecutionError::UnknownStrategyId { .. }
                | crate::commit_strategies::StrategyExecutionError::UnboundStrategyExecutor {
                    ..
                } => ReplayMismatchClass::StrategyExecutorUnavailable,
                _ => ReplayMismatchClass::StrategyExecutionFailure,
            };
            mismatches.push(strategy_mismatch(
                mismatch_class,
                format!("strategy replay re-execution failed: {error:?}"),
                expected_artifacts,
                None,
            ));
            None
        }
    }
}

fn lower_strategy_replay(
    basis_runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    expected_artifacts: &StrategyCommitArtifactBundle,
    execution: &StrategyExecutionDraft,
    mismatches: &mut Vec<ReplayMismatch>,
) -> Option<StrategyCommitArtifactBundle> {
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
            return None;
        }
    };
    let mut strategy_authority = basis_runtime.commit_strategies_authority();
    let lowered = match strategy_authority.lower_execution_with_input(
        basis_runtime,
        &replay_request,
        execution,
        options,
    ) {
        Ok(lowered) => lowered,
        Err(error) => {
            mismatches.push(strategy_mismatch(
                ReplayMismatchClass::StrategyLoweringDrift,
                format!("strategy replay re-lowering failed: {error:?}"),
                expected_artifacts,
                None,
            ));
            return None;
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
            return None;
        }
        match strategy_authority.validate_lowered_plan(basis_runtime, lowered) {
            Ok(validated) => validated.strategy_commit_artifacts().cloned(),
            Err(error) => {
                mismatches.push(strategy_mismatch(
                    ReplayMismatchClass::StrategyLoweringDrift,
                    format!("strategy replay re-validation failed: {error:?}"),
                    expected_artifacts,
                    None,
                ));
                None
            }
        }
    } else {
        Some(StrategyCommitArtifactBundle::from_lowered(
            &lowered,
            &descriptor,
            basis_runtime.runtime_config(),
        ))
    }
}

fn ensure_strategy_replay_basis_branch(
    basis_runtime: &RelationalRuntime,
    envelope: &CanonicalCommitEnvelope,
    expected_artifacts: &StrategyCommitArtifactBundle,
) -> Result<(), String> {
    if envelope.branch_context == BranchId("main".to_string())
        || basis_runtime
            .history()
            .branch_head(&envelope.branch_context)
            .is_some()
    {
        return Ok(());
    }

    let source_branch = expected_artifacts
        .validated_against_commit_id()
        .and_then(|commit_id| {
            basis_runtime
                .replay()
                .canonical_commit_envelope(commit_id)
                .map(|basis| basis.branch_context.clone())
        })
        .unwrap_or_else(|| BranchId("main".to_string()));

    basis_runtime
        .history_authority()
        .fork_branch_from(envelope.branch_context.clone(), &source_branch)
        .map_err(|error| {
            format!(
                "failed to reconstruct strategy replay basis branch {:?} from {:?}: {error:?}",
                envelope.branch_context, source_branch
            )
        })
}
