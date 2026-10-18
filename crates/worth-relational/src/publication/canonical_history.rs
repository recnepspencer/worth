use crate::capabilities::{DurabilityRead, PatchStreamCommitRef, PatchStreamSource};
use crate::history::data::{CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetainedCanonicalEnvelopeGap {
    pub position: PatchStreamPosition,
    pub commit_id: CommitId,
}

pub(crate) fn retained_canonical_envelopes_after(
    runtime: &RelationalRuntime,
    after_position: Option<PatchStreamPosition>,
    max_commits: usize,
) -> Result<Vec<PositionedCanonicalCommit>, RetainedCanonicalEnvelopeGap> {
    let commit_refs = runtime.patch_stream_commits_after(after_position, max_commits);
    resolve_retained_commit_refs(runtime, &commit_refs)
}

pub(crate) fn retained_canonical_envelope_at_position(
    runtime: &RelationalRuntime,
    position: PatchStreamPosition,
    preloaded_durable_envelopes: Option<&[PositionedCanonicalCommit]>,
) -> Option<PositionedCanonicalCommit> {
    runtime
        .history
        .positioned_canonical_commit_at_patch(position)
        .map(|commit| commit.as_ref().clone())
        .or_else(|| {
            let durable = preloaded_durable_envelopes
                .map(|envelopes| envelopes.to_vec())
                .unwrap_or_else(|| durable_canonical_envelopes(runtime));
            durable
                .into_iter()
                .find(|commit| commit.position() == position)
        })
}

pub(crate) fn durable_canonical_envelopes(
    runtime: &RelationalRuntime,
) -> Vec<PositionedCanonicalCommit> {
    if runtime.durable_checkpoints().is_empty() && runtime.durable_log().is_empty() {
        return Vec::new();
    }

    // Request-free observers borrow already-admitted native custody. Disk
    // readmission belongs exclusively to execution-taking recovery planning.
    let mut envelopes = runtime
        .durable_checkpoints()
        .last()
        .map(|checkpoint| checkpoint.envelopes.clone())
        .unwrap_or_default();
    envelopes.extend(
        runtime
            .durable_log()
            .into_iter()
            .map(|commit| commit.as_ref().clone()),
    );
    envelopes.sort_by_key(|commit| commit.position());
    envelopes.dedup_by_key(|commit| commit.position());
    envelopes
}

fn resolve_retained_commit_refs(
    runtime: &RelationalRuntime,
    commit_refs: &[PatchStreamCommitRef],
) -> Result<Vec<PositionedCanonicalCommit>, RetainedCanonicalEnvelopeGap> {
    let mut durable_envelopes = None;
    let mut envelopes = Vec::with_capacity(commit_refs.len());

    for commit_ref in commit_refs {
        if let Some(commit) = runtime
            .history
            .positioned_canonical_commit(commit_ref.commit_id)
            .map(|commit| commit.as_ref().clone())
        {
            envelopes.push(commit);
            continue;
        }
        let durable = durable_envelopes.get_or_insert_with(|| durable_canonical_envelopes(runtime));
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
