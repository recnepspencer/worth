//! Streaming append-only build of a paged commit index.
//!
//! The builder buffers at most one page. Resuming from a prior index carries
//! every prior page except the last unchanged (`O(touched)` capture); the
//! caller re-pushes the last page's rows so an appended tail fills it, and an
//! unchanged history re-encodes the same content-addressed member.
use std::collections::BTreeSet;

use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use super::page::CommitIndexPage;
use super::{CheckpointCommitIndex, CommitIndexPageRef, COMMIT_PAGE_BYTES, COMMIT_PAGE_ENTRIES};
use crate::history::commit::index_entry::RelationalCommitIndexEntry;
use crate::history::data::CommitId;

/// Frame, page header and array headers of one encoded page.
const PAGE_OVERHEAD: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitIndexBuildError {
    /// Rows must arrive at consecutive positions starting after the carry.
    PositionOutOfOrder,
    /// One row alone exceeds a page's byte budget.
    RowTooLarge,
    Member(Cause),
}

pub struct CommitIndexBuilder<'s> {
    sink: &'s mut dyn ChunkMemberSink,
    pages: Vec<CommitIndexPageRef>,
    rows: Vec<RelationalCommitIndexEntry>,
    bytes: usize,
    next: u64,
    tail: Option<RelationalCommitIndexEntry>,
    wanted: BTreeSet<CommitId>,
    heads: Vec<RelationalCommitIndexEntry>,
}

impl<'s> CommitIndexBuilder<'s> {
    /// Starts after `carried`, a prefix of a prior index's pages; `wanted`
    /// names the selected roots whose pushed entries become inline heads.
    pub fn resume(
        sink: &'s mut dyn ChunkMemberSink,
        carried: &[CommitIndexPageRef],
        wanted: BTreeSet<CommitId>,
    ) -> Result<Self, CommitIndexBuildError> {
        let next = carried
            .last()
            .map_or(Some(1), CommitIndexPageRef::end)
            .ok_or(CommitIndexBuildError::PositionOutOfOrder)?;
        Ok(Self {
            sink,
            pages: carried.to_vec(),
            rows: Vec::new(),
            bytes: 0,
            next,
            tail: None,
            wanted,
            heads: Vec::new(),
        })
    }

    pub fn push(&mut self, row: RelationalCommitIndexEntry) -> Result<(), CommitIndexBuildError> {
        if row.position.0 != self.next {
            return Err(CommitIndexBuildError::PositionOutOfOrder);
        }
        let size = rmp_serde::encode::to_vec(&row)
            .map_err(|_| CommitIndexBuildError::Member(Cause::Damaged))?
            .len();
        if size + PAGE_OVERHEAD > COMMIT_PAGE_BYTES {
            return Err(CommitIndexBuildError::RowTooLarge);
        }
        if self.rows.len() == COMMIT_PAGE_ENTRIES as usize
            || self.bytes + size + PAGE_OVERHEAD > COMMIT_PAGE_BYTES
        {
            self.flush()?;
        }
        self.next = self
            .next
            .checked_add(1)
            .ok_or(CommitIndexBuildError::PositionOutOfOrder)?;
        if self.wanted.remove(&row.commit.commit_id) {
            self.heads.push(row.clone());
        }
        self.bytes += size;
        self.tail = Some(row.clone());
        self.rows.push(row);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CommitIndexBuildError> {
        let Some(first) = self.rows.first().map(|row| row.position.0) else {
            return Ok(());
        };
        let rows = std::mem::take(&mut self.rows);
        let range = |value: fn(&RelationalCommitIndexEntry) -> u64| {
            let low = rows.iter().map(value).min().unwrap_or(0);
            [low, rows.iter().map(value).max().unwrap_or(0)]
        };
        let commits = range(|row| row.commit.commit_id.0);
        let versions = range(|row| row.commit.version_id.0);
        let entries = rows.len() as u32;
        let member = encode_member(
            ChunkMemberKind::CommitIndexPage,
            &CommitIndexPage { first, rows },
            COMMIT_PAGE_BYTES,
        )
        .map_err(CommitIndexBuildError::Member)?;
        let locator = member.locator;
        self.sink
            .put(locator, member.bytes)
            .map_err(CommitIndexBuildError::Member)?;
        self.pages.push(CommitIndexPageRef {
            first,
            entries,
            commits,
            versions,
            member: locator,
        });
        self.bytes = 0;
        Ok(())
    }

    /// Selected roots not pushed since the carry; the caller resolves them
    /// from the prior index before [`Self::finish`].
    pub fn unresolved_heads(&self) -> impl Iterator<Item = CommitId> + '_ {
        self.wanted.iter().copied()
    }

    /// Writes the last page. `carried` supplies the prior tail when nothing
    /// was pushed and the entries of roots that lie in carried pages.
    pub fn finish(
        mut self,
        prior_tail: Option<RelationalCommitIndexEntry>,
        carried_heads: Vec<RelationalCommitIndexEntry>,
    ) -> Result<CheckpointCommitIndex, CommitIndexBuildError> {
        self.flush()?;
        let mut heads = self.heads;
        heads.extend(carried_heads);
        heads.sort_unstable_by_key(|entry| entry.position);
        Ok(CheckpointCommitIndex {
            count: self.next - 1,
            tail: self.tail.or(prior_tail),
            heads,
            pages: self.pages,
        })
    }
}
