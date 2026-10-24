use crate::capabilities::{CommitEnvelopeSource, DurabilityRead, RuntimeConfigSource};
use crate::durability::data::RecoveryPlan;
use crate::history::data::CommitId;
use crate::schema::data::DescriptorSemanticsVersion;

pub(in crate::replay) fn replay_recovery_plan_for_chain(
    source: &crate::runtime::RelationalRuntime,
    chain: &[CommitId],
    verification_mode: crate::replay::data::ReplayVerificationMode,
) -> RecoveryPlan {
    let checkpoint = select_replay_checkpoint(source, chain);
    let tail_log = replay_tail_log(source, chain, checkpoint.as_ref());
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
    RecoveryPlan::new(
        source.runtime_config().clone(),
        checkpoint,
        tail_log,
        authority_continuity,
        replay_verification_mode_to_recovery_mode(verification_mode),
        target_descriptor_semantics_version(source, chain),
        restore_authoritative_envelope_commit_ids,
    )
    .with_commit_strategy_executors(source.commit_strategy_executor_registry().clone())
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
