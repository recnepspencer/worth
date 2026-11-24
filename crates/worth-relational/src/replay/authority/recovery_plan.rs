use crate::branch::transition::RelationalBranchTransitionSequence;
use crate::capabilities::{DurabilityRead, RuntimeConfigSource};
use crate::durability::data::RecoveryPlan;
use crate::history::data::CommitId;
use crate::replay::planning::{
    authoritative_replay_envelopes, replay_fork_transitions, replay_tail_log,
    select_replay_checkpoint, target_descriptor_semantics_version,
};

pub(in crate::replay) fn replay_recovery_plan_for_chain(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
    verification_mode: crate::replay::data::ReplayVerificationMode,
) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
    let checkpoint = select_replay_checkpoint(source.durable_checkpoints(), chain);
    let tail_log = replay_tail_log(source.history.canonical_read(), chain, checkpoint.as_ref());
    let covered = checkpoint
        .as_ref()
        .map_or(RelationalBranchTransitionSequence(0), |checkpoint| {
            checkpoint.coverage.up_to_branch_transition
        });
    let transitions = source
        .durability()
        .reconstruct_branch_transition_history(execution, covered)?;
    let branch_transitions = replay_fork_transitions(transitions, &tail_log);
    let restore_authoritative_envelope_commit_ids =
        authoritative_replay_envelopes(source.history.canonical_read(), chain);
    let checkpoint_envelopes = checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.as_slice())
        .unwrap_or(&[]);
    let authority_continuity =
        crate::runtime::durability_access::authority_continuity_for_envelopes(
            source,
            checkpoint_envelopes,
            &tail_log,
        );
    Ok(RecoveryPlan::new(
        source.runtime_config().clone(),
        checkpoint,
        tail_log,
        branch_transitions,
        authority_continuity,
        replay_verification_mode_to_recovery_mode(verification_mode),
        target_descriptor_semantics_version(source.history.canonical_read(), chain),
        restore_authoritative_envelope_commit_ids,
        source.durability.cold_source(),
    )
    .with_commit_strategy_executors(source.commit_strategy_executor_registry().clone()))
}

fn replay_verification_mode_to_recovery_mode(
    mode: crate::replay::data::ReplayVerificationMode,
) -> crate::durability::data::RecoveryVerificationMode {
    match mode {
        crate::replay::data::ReplayVerificationMode::NormalRecoveryVerification => {
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification
        }
        crate::replay::data::ReplayVerificationMode::AuditRecoveryVerification => {
            crate::durability::data::RecoveryVerificationMode::AuditRecoveryVerification
        }
        crate::replay::data::ReplayVerificationMode::CorruptionDiagnosisReplay => {
            crate::durability::data::RecoveryVerificationMode::CorruptionDiagnosisReplay
        }
    }
}
