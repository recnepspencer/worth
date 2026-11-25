use std::collections::BTreeSet;

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::history::envelope_demand::CanonicalHistoryRead;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::replay::data::replay_errors::ReplayFailureClass;

pub fn load_replay_envelope(
    source: CanonicalHistoryRead<'_>,
    commit_id: CommitId,
) -> Option<std::sync::Arc<CanonicalCommitEnvelope>> {
    source.canonical_envelope(commit_id)
}

pub fn replay_commit_closure_by_commit_id_order(
    history: CanonicalHistoryRead<'_>,
    commit_id: CommitId,
) -> ReplayCommitClosure {
    let mut ordered = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut nodes_visited = 0usize;
    let mut parent_checks = 0usize;
    let result = visit_replay_commit_closure(
        history,
        commit_id,
        &mut visiting,
        &mut ordered,
        &mut nodes_visited,
        &mut parent_checks,
    );
    ReplayCommitClosure {
        commits: result.map(|_| ordered),
        nodes_visited,
        parent_checks,
    }
}

pub struct ReplayCommitClosure {
    pub commits: Result<Vec<CommitId>, ReplayFailureClass>,
    pub nodes_visited: usize,
    pub parent_checks: usize,
}

fn visit_replay_commit_closure(
    history: CanonicalHistoryRead<'_>,
    commit_id: CommitId,
    visiting: &mut BTreeSet<CommitId>,
    ordered: &mut Vec<CommitId>,
    nodes_visited: &mut usize,
    parent_checks: &mut usize,
) -> Result<(), ReplayFailureClass> {
    if ordered.contains(&commit_id) {
        return Ok(());
    }
    let Some(envelope) = history.canonical_envelope(commit_id) else {
        return Err(ReplayFailureClass::MissingAuthoritativeParentClosure);
    };
    if !visiting.insert(commit_id) {
        return Err(ReplayFailureClass::MissingAuthoritativeParentClosure);
    }
    *nodes_visited += 1;
    // Replay walks parents in authoritative published order. This traversal
    // must not normalize, sort, or reinterpret the post-publication parent
    // list.
    for parent in envelope.commit.ordered_parents().as_slice() {
        *parent_checks += 1;
        visit_replay_commit_closure(
            history,
            *parent,
            visiting,
            ordered,
            nodes_visited,
            parent_checks,
        )?;
    }
    visiting.remove(&commit_id);
    ordered.push(commit_id);
    Ok(())
}
