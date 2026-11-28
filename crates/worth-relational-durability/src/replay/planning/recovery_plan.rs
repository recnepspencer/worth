use worth_relational_branch::internal_port::history::envelope_demand::CanonicalHistoryRead;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;

pub fn select_replay_checkpoint(
    checkpoints: Vec<std::sync::Arc<crate::durability::data::DurableCheckpoint>>,
    chain: &[CommitId],
) -> Option<crate::durability::data::DurableCheckpoint> {
    checkpoints
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

pub fn replay_tail_log(
    history: CanonicalHistoryRead<'_>,
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
            history.performed.positioned(commit_id).map(|commit| {
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
pub fn replay_fork_transitions(
    transitions: Vec<worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition>,
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
) -> crate::durability::data::RecoveryBranchTransitions {
    let mut needed = tail_log
        .iter()
        .map(|commit| commit.envelope().branch_context.clone())
        .collect::<std::collections::BTreeSet<_>>();
    // Sources precede their targets. Walking backwards closes the selection
    // over source forks, including intermediate branches with no own commits.
    let mut forks = transitions
        .into_iter()
        .rev()
        .filter_map(|transition| {
            if let worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition::Fork {
                target,
                source,
                ..
            }
            | worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition::RetainedFork {
                target,
                source,
                ..
            } = transition.transition()
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
    crate::durability::data::RecoveryBranchTransitions::Selected(forks)
}

pub fn authoritative_replay_envelopes(chain: &[CommitId]) -> Vec<CommitId> {
    let target_commit_id = chain.last().copied();
    let mut restore_authoritative_envelope_commit_ids = chain
        .iter()
        .copied()
        .filter(|commit_id| Some(*commit_id) != target_commit_id)
        .collect::<Vec<_>>();
    restore_authoritative_envelope_commit_ids.sort_unstable();
    restore_authoritative_envelope_commit_ids
}

pub fn target_descriptor_semantics_version(
    history: CanonicalHistoryRead<'_>,
    chain: &[CommitId],
) -> DescriptorSemanticsVersion {
    chain
        .last()
        .and_then(|commit_id| history.canonical_envelope(*commit_id))
        .map(|envelope| envelope.descriptor_semantics_version)
        .unwrap_or_default()
}
