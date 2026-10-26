use crate::capabilities::{CommitEnvelopeSource, DurabilityRead, RuntimeConfigSource};
use crate::durability::data::RecoveryPlan;
use crate::history::data::CommitId;
use crate::schema::data::DescriptorSemanticsVersion;

pub(in crate::replay) fn replay_recovery_plan_for_chain(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
    verification_mode: crate::replay::data::ReplayVerificationMode,
) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
    let checkpoint = select_replay_checkpoint(source, chain);
    let tail_log = replay_tail_log(source, chain, checkpoint.as_ref());
    let branch_transitions =
        replay_fork_transitions(execution, source, &tail_log, checkpoint.as_ref())?;
    let restore_authoritative_envelope_commit_ids = authoritative_replay_envelopes(source, chain);
    let checkpoint_envelopes = checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.as_slice())
        .unwrap_or(&[]);
    let authority_continuity = crate::durability::access::authority_continuity_for_envelopes(
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
        target_descriptor_semantics_version(source, chain),
        restore_authoritative_envelope_commit_ids,
        source.durability.cold_source(),
    )
    .with_commit_strategy_executors(source.commit_strategy_executor_registry().clone()))
}

fn select_replay_checkpoint(
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
) -> Option<crate::durability::data::DurableCheckpoint> {
    source
        .durable_checkpoints()
        .into_iter()
        .rev()
        .find(|checkpoint| {
            checkpoint
                .coverage
                .up_to_commit
                .as_ref()
                .map(|commit| chain.contains(&commit.commit_id))
                .unwrap_or(false)
        })
        .map(|checkpoint| checkpoint.as_ref().clone())
}

fn replay_tail_log(
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
    checkpoint: Option<&crate::durability::data::DurableCheckpoint>,
) -> Vec<crate::durability::migration::ReadmittedCanonicalCommit> {
    let tail_start = checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.coverage.up_to_commit.as_ref())
        .map(|commit| commit.commit_id);
    chain
        .iter()
        .copied()
        .filter(|commit_id| tail_start.is_none_or(|start| *commit_id > start))
        .filter_map(|commit_id| {
            source
                .history
                .positioned_canonical_commit(commit_id)
                .map(|commit| {
                    crate::durability::migration::ReadmittedCanonicalCommit::exact(
                        commit.as_ref().clone(),
                    )
                })
        })
        .collect()
}

/// The forks a partial chain needs: every fork the source performed after the
/// checkpoint's coverage needed by a chain branch or any of its fork sources.
/// A fork is the only source of a branch's existence, so a chain commit on a
/// forked branch replays after its fork.
fn replay_fork_transitions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::runtime::RelationalRuntime,
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
    checkpoint: Option<&crate::durability::data::DurableCheckpoint>,
) -> Result<
    crate::durability::data::RecoveryBranchTransitions,
    crate::durability::data::RelationalRecoveryStop,
> {
    let covered = checkpoint.map_or(
        crate::branch::RelationalBranchTransitionSequence(0),
        |checkpoint| checkpoint.coverage.up_to_branch_transition,
    );
    let mut needed = tail_log
        .iter()
        .map(|commit| commit.envelope().branch_context.clone())
        .collect::<std::collections::BTreeSet<_>>();
    // Sources precede their targets. Walking backwards closes the selection
    // over source forks, including intermediate branches with no own commits.
    let mut forks = source
        .durability()
        .reconstruct_branch_transition_history(execution, covered)?
        .into_iter()
        .rev()
        .filter_map(|transition| {
            if let crate::branch::RelationalBranchTransition::Fork { target, source, .. } =
                transition.transition()
                && needed.contains(target)
            {
                needed.insert(source.clone());
                Some(transition)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    forks.reverse();
    Ok(crate::durability::data::RecoveryBranchTransitions::Selected(forks))
}

fn authoritative_replay_envelopes(
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
) -> Vec<CommitId> {
    let target_commit_id = chain.last().copied();
    let mut restore_authoritative_envelope_commit_ids = chain
        .iter()
        .copied()
        .filter(|commit_id| Some(*commit_id) != target_commit_id)
        .chain(
            chain
                .iter()
                .copied()
                .filter_map(|commit_id| source.canonical_envelope_owned(commit_id))
                .filter(|envelope| envelope.strategy_artifacts.is_some())
                .map(|envelope| envelope.commit.commit_id),
        )
        .collect::<Vec<_>>();
    restore_authoritative_envelope_commit_ids.sort_unstable();
    restore_authoritative_envelope_commit_ids
}

fn target_descriptor_semantics_version(
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
) -> DescriptorSemanticsVersion {
    chain
        .last()
        .and_then(|commit_id| source.canonical_envelope_owned(*commit_id))
        .map(|envelope| envelope.descriptor_semantics_version)
        .unwrap_or_default()
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
