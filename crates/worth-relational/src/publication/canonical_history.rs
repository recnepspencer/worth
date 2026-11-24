use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::stream_commit_ref::PatchStreamCommitRef;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::state::subsystems::durability::commit_log::read_coverage::DurableCanonicalCoverage;
use crate::runtime::state::subsystems::history::HistorySubsystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetainedCanonicalEnvelopeGap {
    pub position: PatchStreamPosition,
    pub commit_id: CommitId,
}

pub(crate) fn retained_canonical_envelopes_after(
    history: &HistorySubsystem,
    durability: DurableCanonicalCoverage<'_>,
    after_position: Option<PatchStreamPosition>,
    max_commits: usize,
) -> Result<Vec<PositionedCanonicalCommit>, RetainedCanonicalEnvelopeGap> {
    let commit_refs = patch_stream_commits_after(history, after_position, max_commits);
    resolve_retained_commit_refs(history, durability, &commit_refs)
}

pub(crate) fn retained_canonical_envelope_at_position(
    history: &HistorySubsystem,
    durability: DurableCanonicalCoverage<'_>,
    position: PatchStreamPosition,
    preloaded_durable_envelopes: Option<&[PositionedCanonicalCommit]>,
) -> Option<PositionedCanonicalCommit> {
    history
        .positioned_canonical_commit_at_patch(position)
        .map(|commit| commit.as_ref().clone())
        .or_else(|| {
            let durable = preloaded_durable_envelopes
                .map(|envelopes| envelopes.to_vec())
                .unwrap_or_else(|| durable_canonical_envelopes(durability));
            durable
                .into_iter()
                .find(|commit| commit.position() == position)
        })
}

pub(crate) fn durable_canonical_envelopes(
    durability: DurableCanonicalCoverage<'_>,
) -> Vec<PositionedCanonicalCommit> {
    if durability.checkpoints().is_empty() && durability.log().is_empty() {
        return Vec::new();
    }

    // Request-free observers borrow already-admitted native custody. Disk
    // readmission belongs exclusively to execution-taking recovery planning.
    let mut envelopes = durability
        .checkpoints()
        .last()
        .map(|checkpoint| checkpoint.envelopes.clone())
        .unwrap_or_default();
    envelopes.extend(
        durability
            .log()
            .into_iter()
            .map(|commit| commit.as_ref().clone()),
    );
    envelopes.sort_by_key(|commit| commit.position());
    envelopes.dedup_by_key(|commit| commit.position());
    envelopes
}

fn resolve_retained_commit_refs(
    history: &HistorySubsystem,
    durability: DurableCanonicalCoverage<'_>,
    commit_refs: &[PatchStreamCommitRef],
) -> Result<Vec<PositionedCanonicalCommit>, RetainedCanonicalEnvelopeGap> {
    let mut durable_envelopes = None;
    let mut envelopes = Vec::with_capacity(commit_refs.len());

    for commit_ref in commit_refs {
        if let Some(commit) = history
            .positioned_canonical_commit(commit_ref.commit_id)
            .map(|commit| commit.as_ref().clone())
        {
            envelopes.push(commit);
            continue;
        }
        let durable =
            durable_envelopes.get_or_insert_with(|| durable_canonical_envelopes(durability));
        if let Some(envelope) = durable
            .iter()
            .find(|commit| commit.position() == commit_ref.position)
            .cloned()
        {
            envelopes.push(envelope);
            continue;
        }

        return Err(RetainedCanonicalEnvelopeGap {
            position: commit_ref.position,
            commit_id: commit_ref.commit_id,
        });
    }

    Ok(envelopes)
}

pub(crate) fn patch_stream_commits_after(
    history: &HistorySubsystem,
    after_position: Option<PatchStreamPosition>,
    max_commits: usize,
) -> Vec<PatchStreamCommitRef> {
    let start = after_position
        .map(std::ops::Bound::Excluded)
        .unwrap_or(std::ops::Bound::Unbounded);
    let mut entries = history
        .recorded_patch_positions_after(after_position, max_commits)
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    entries.extend(history.canonical_patch_routes_after(after_position, max_commits));
    entries
        .range((start, std::ops::Bound::Unbounded))
        .map(|(position, commit_id)| PatchStreamCommitRef {
            position: *position,
            commit_id: *commit_id,
        })
        .take(max_commits)
        .collect()
}
