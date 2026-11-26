//! `CHK24` capture of the commit index as content-addressed page members.
//!
//! The index is append-only, so every prior page except the last covers an
//! unchanged prefix and is carried as-is (`O(touched)` capture): only the
//! last prior page and the appended tail are encoded. A carried page the
//! destination does not hold is copied by re-encoding its faulted rows,
//! which reproduces its exact member.
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::durability::data::{
    CheckpointCommitIndex, DurabilityError, RecoveryFailureClass, RelationalCheckpointMember,
};
use worth_execution::ExecutionImmutableBytes;
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry as Entry;
use worth_relational_branch::internal_port::history::commit::paged_index::builder::CommitIndexBuilder;
use worth_relational_branch::internal_port::history::commit::paged_index::page::CommitIndexPage;
use worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex;
use worth_relational_branch::internal_port::history::commit::paged_index::COMMIT_PAGE_BYTES;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator as Locator;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};

fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}

fn unavailable(cause: impl std::fmt::Debug) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::ColdStorageUnavailable,
        format!("prior commit index page unavailable: {cause:?}"),
    )
}

/// The first position a capture over `prior` re-pages: the start of its last
/// page, since every earlier page is carried unchanged.
pub(crate) fn carry_start(prior: Option<&CheckpointCommitIndex>) -> u64 {
    prior
        .and_then(|prior| prior.pages.last())
        .map_or(1, |page| page.first)
}

/// The rows of `prior`'s last page: the only prior page a capture reads.
pub(crate) fn tail_rows(prior: &ColdCommitIndex) -> Result<Vec<Entry>, DurabilityError> {
    match prior.descriptor().pages.len().checked_sub(1) {
        Some(last) => Ok(prior.page(last).map_err(unavailable)?.page.rows.clone()),
        None => Ok(Vec::new()),
    }
}

/// Where a capture may carry pages instead of writing them.
pub(crate) enum CommitPageCarry<'a> {
    /// The destination's exact members: any page it holds is carried.
    Held(&'a BTreeSet<Locator>),
    /// No membership is known: carry the prior index's pages, or nothing.
    Prior(bool),
}

/// Pages `rows`, the index rows from [`carry_start`] onward. `prior`'s
/// earlier pages are retained; `carry` says which pages the destination
/// already holds. Selected roots outside `rows` resolve from the prior index
/// (inline heads answer without a page read).
pub(crate) fn page_commit_rows(
    prior: Option<&ColdCommitIndex>,
    rows: &[Entry],
    roots: &BTreeSet<CommitId>,
    carry: CommitPageCarry<'_>,
) -> Result<(CheckpointCommitIndex, Vec<RelationalCheckpointMember>), DurabilityError> {
    let descriptor = prior.map(|prior| prior.descriptor().as_ref());
    let carried = descriptor.map_or(&[][..], |prior| {
        &prior.pages[..prior.pages.len().saturating_sub(1)]
    });
    let mut sink = CollectedMembers::default();
    let mut builder = CommitIndexBuilder::resume(&mut sink, carried, roots.clone())
        .map_err(|cause| corrupt(format!("commit index carry: {cause:?}")))?;
    for row in rows {
        builder
            .push(row.clone())
            .map_err(|cause| corrupt(format!("commit index page: {cause:?}")))?;
    }
    let mut carried_heads = Vec::new();
    for commit in builder.unresolved_heads().collect::<Vec<_>>() {
        let head = prior
            .map(|prior| prior.by_commit(commit))
            .transpose()
            .map_err(unavailable)?
            .flatten()
            .ok_or_else(|| corrupt("a selected root has no indexed commit"))?;
        carried_heads.push(head);
    }
    let prior_tail = descriptor.and_then(|prior| prior.tail.clone());
    let index = builder
        .finish(prior_tail, carried_heads)
        .map_err(|cause| corrupt(format!("commit index page: {cause:?}")))?;
    let written = sink
        .members
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let prior_pages = descriptor
        .map(|prior| prior.page_members().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let mut members = Vec::with_capacity(index.pages.len());
    for (ordinal, page) in index.pages.iter().enumerate() {
        let locator = page.member;
        let held = match carry {
            CommitPageCarry::Held(members) => members.contains(&locator),
            CommitPageCarry::Prior(carry) => carry && prior_pages.contains(&locator),
        };
        let member = if held {
            RelationalCheckpointMember::Carried { locator }
        } else if let Some(bytes) = written.get(&locator) {
            RelationalCheckpointMember::New {
                locator,
                bytes: immutable(bytes.clone()),
            }
        } else {
            let prior = prior.ok_or_else(|| corrupt("commit page has no source"))?;
            let rows = prior.page(ordinal).map_err(unavailable)?.page.rows.clone();
            let encoded = encode_member(
                ChunkMemberKind::CommitIndexPage,
                &CommitIndexPage {
                    first: page.first,
                    rows,
                },
                COMMIT_PAGE_BYTES,
            )
            .map_err(|cause| corrupt(format!("commit index page copy: {cause:?}")))?;
            if encoded.locator != locator {
                return Err(corrupt("carried commit page differs from its rows"));
            }
            RelationalCheckpointMember::New {
                locator,
                bytes: immutable(encoded.bytes),
            }
        };
        members.push(member);
    }
    Ok((index, members))
}

fn immutable(bytes: Vec<u8>) -> ExecutionImmutableBytes {
    ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice()))
}

/// Every entry of `directory`'s index, in position order. Only the complete
/// original-input capture, which replays every commit, reads them all.
pub(crate) fn prior_rows(
    directory: &crate::durability::data::RelationalCheckpointDirectory,
) -> Result<Vec<Entry>, crate::durability::data::RelationalRecoveryStop> {
    let mut rows = Vec::with_capacity(directory.manifest.commits.len());
    directory.for_each_commit(|row| {
        rows.push(row.clone());
        Ok::<_, crate::durability::data::RelationalRecoveryStop>(())
    })?;
    Ok(rows)
}
