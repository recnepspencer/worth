use std::collections::BTreeSet;

use crate::capabilities::CommitEnvelopeSource;
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::replay::data::ReplayFailureClass;
use crate::runtime::RelationalRuntime;

pub(in crate::replay) fn load_replay_envelope(
    source: &impl CommitEnvelopeSource,
    commit_id: CommitId,
) -> Option<CanonicalCommitEnvelope> {
    #[cfg(test)]
    replay_contacts::observe();
    source.resident_commit_envelope(commit_id)
}

pub(in crate::replay) fn replay_commit_closure_by_commit_id_order(
    runtime: &RelationalRuntime,
    history: &impl CommitEnvelopeSource,
    commit_id: CommitId,
) -> Result<Vec<CommitId>, ReplayFailureClass> {
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
    runtime
        .performance_access()
        .count_merge_history_replay_planning(nodes_visited, parent_checks);
    result.map(|_| ordered)
}

fn visit_replay_commit_closure(
    history: &impl CommitEnvelopeSource,
    commit_id: CommitId,
    visiting: &mut BTreeSet<CommitId>,
    ordered: &mut Vec<CommitId>,
    nodes_visited: &mut usize,
    parent_checks: &mut usize,
) -> Result<(), ReplayFailureClass> {
    if ordered.contains(&commit_id) {
        return Ok(());
    }
    let Some(envelope) = history.resident_commit_envelope(commit_id) else {
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

#[cfg(test)]
pub(in crate::replay) mod replay_contacts {
    use std::cell::Cell;
    thread_local! { static CONTACTS: Cell<usize> = const { Cell::new(0) }; }
    pub(super) fn observe() {
        CONTACTS.with(|count| count.set(count.get() + 1));
    }
    pub(in crate::replay) fn count() -> usize {
        CONTACTS.with(Cell::get)
    }
}
