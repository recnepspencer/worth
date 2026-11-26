//! One authenticated commit-index page and its check against its descriptor.
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    decode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::page_cache::ResidentPage;

use super::{CheckpointCommitIndex, CommitIndexPageRef, COMMIT_PAGE_ENTRIES};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::commit::index_entry::RelationalCommitIndexEntry;

/// The encoded page: rows at consecutive positions starting at `first`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitIndexPage {
    pub first: u64,
    pub rows: Vec<RelationalCommitIndexEntry>,
}

/// A decoded resident page and the encoded size that bounds its heap.
#[derive(Debug)]
pub struct ResidentCommitPage {
    pub page: CommitIndexPage,
    encoded: u64,
}

impl CommitIndexPage {
    /// Rows are non-empty, within budget, at consecutive positions from
    /// `first`, and carry non-zero commit ids.
    fn is_canonical(&self) -> bool {
        !self.rows.is_empty()
            && self.rows.len() <= COMMIT_PAGE_ENTRIES as usize
            && self.rows.iter().enumerate().all(|(offset, row)| {
                row.commit.commit_id.0 != 0
                    && self.first.checked_add(offset as u64) == Some(row.position.0)
            })
    }
}

impl ResidentPage for ResidentCommitPage {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        let page: CommitIndexPage = decode_member(ChunkMemberKind::CommitIndexPage, bytes, locator)?;
        if !page.is_canonical() {
            return Err(Cause::Damaged);
        }
        Ok(Self {
            page,
            encoded: bytes.len() as u64,
        })
    }

    fn resident_bytes(&self) -> u64 {
        // Row heaps (parents, branch names, root descriptors) are bounded by
        // their encoded bytes; the row array is counted at its capacity.
        let rows = self.page.rows.capacity() * std::mem::size_of::<RelationalCommitIndexEntry>();
        (std::mem::size_of::<Self>() + rows) as u64 + self.encoded
    }
}

impl ResidentCommitPage {
    /// Checks a faulted page against the manifest's descriptor and the inline
    /// entries it must reproduce. Pages are immutable and content-addressed,
    /// so a page that passes once passes on every later hit.
    pub fn admit(
        &self,
        reference: &CommitIndexPageRef,
        index: &CheckpointCommitIndex,
    ) -> Result<(), Cause> {
        let rows = &self.page.rows;
        if self.page.first != reference.first || rows.len() != reference.entries as usize {
            return Err(Cause::IntegrityMismatch);
        }
        let range = |values: &mut dyn Iterator<Item = u64>| {
            values.fold(None, |range: Option<[u64; 2]>, value| {
                Some(range.map_or([value, value], |[low, high]| {
                    [low.min(value), high.max(value)]
                }))
            })
        };
        let commits = range(&mut rows.iter().map(|row| row.commit.commit_id.0));
        let versions = range(&mut rows.iter().map(|row| row.commit.version_id.0));
        if commits != Some(reference.commits) || versions != Some(reference.versions) {
            return Err(Cause::IntegrityMismatch);
        }
        for row in rows {
            if row
                .commit
                .parents
                .iter()
                .any(|parent| *parent == row.commit.commit_id)
                || index
                    .inline_at(row.position.0)
                    .is_some_and(|inline| inline != row)
            {
                return Err(Cause::IntegrityMismatch);
            }
        }
        Ok(())
    }

    pub fn row(&self, position: u64) -> Option<&RelationalCommitIndexEntry> {
        let offset = usize::try_from(position.checked_sub(self.page.first)?).ok()?;
        self.page.rows.get(offset)
    }
}
