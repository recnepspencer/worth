//! The commit rows one capture pages: the prior's last page and the tail.
use super::*;
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex;

pub(super) type PriorCommits = Option<Arc<ColdCommitIndex>>;

/// Rows from the prior's last page onward, merged with resident journal
/// entries and envelopes above the covered prefix; returns them with the
/// prior index and its covered count.
pub(super) fn capture(
    checkpoint: &DurableCheckpoint,
    prior: Option<&RelationalCheckpointDirectory>,
    journal_index: Vec<Entry>,
    control: &mut super::super::CheckpointEncodingControl<'_, '_, '_, '_, '_>,
    original_digest: impl Fn(
        contracts::publication::patch::data::patch_position::PatchStreamPosition,
    ) -> Result<[u8; 32], RelationalNativeCheckpointCaptureDenial>,
) -> Result<(Vec<Entry>, PriorCommits, u64), RelationalNativeCheckpointCaptureDenial> {
    let mut commits = BTreeMap::new();
    // G7: only the prior's last page is re-paged; covered rows below it
    // are carried, so resident entries at covered positions are skipped.
    let prior_commits = prior.map(|prior| prior.commit_index().clone());
    let covered = prior_commits.as_ref().map_or(0, |index| index.count());
    let tail = match &prior_commits {
        Some(index) => super::super::commit_pages::tail_rows(index)?,
        None => Vec::new(),
    };
    for entry in tail.into_iter().chain(
        journal_index
            .into_iter()
            .filter(|entry| entry.position.0 > covered),
    ) {
        if let Some(existing) = commits.insert(entry.position, entry.clone())
            && existing != entry
        {
            return Err(corrupt("checkpoint commit index has competing placement facts").into());
        }
    }
    for positioned in &checkpoint.envelopes {
        control.checkpoint(0)?;
        if positioned.position().0 <= covered && !commits.contains_key(&positioned.position()) {
            continue;
        }
        let envelope = positioned.envelope();
        let root = checkpoint
            .commit_descriptors
            .get(&envelope.commit.commit_id)
            .ok_or_else(|| corrupt("checkpoint commit lacks its catalog descriptor"))?;
        if let Some(existing) = commits.get(&positioned.position()) {
            if existing.commit != envelope.commit
                || existing.association_discovery != envelope.association_discovery()
                || &existing.root != root
                || existing.schema_version != envelope.schema_version
            {
                return Err(corrupt("resident envelope disagrees with its admitted index").into());
            }
        } else {
            let payload_digest = original_digest(positioned.position())?;
            commits.insert(
                positioned.position(),
                worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry {
                    association_discovery: envelope.association_discovery(),
                    commit: envelope.commit.clone(),
                    position: positioned.position(),
                    root: root.clone(),
                    payload_digest,
                    schema_version: envelope.schema_version,
                },
            );
        }
    }
    let commits = commits.into_values().collect::<Vec<_>>();
    Ok((commits, prior_commits, covered))
}
