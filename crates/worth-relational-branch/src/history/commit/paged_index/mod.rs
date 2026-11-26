//! `CHK24` paged checkpoint commit index (`G7`).
//!
//! A checkpoint's commit index is append-only in patch-stream order: entry
//! `i` sits at position `i + 1`. The entries live in bounded content-addressed
//! pages; the manifest retains only [`CheckpointCommitIndex`]: the count, the
//! tail and selected-head entries reopen needs without reading a page, and one
//! fixed-size [`CommitIndexPageRef`] per page. Reopen validates it read-free;
//! lookups fault only the pages they touch (see [`reader::ColdCommitIndex`]).
use serde::{Deserialize, Serialize};

use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::commit::index_entry::RelationalCommitIndexEntry;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;

pub mod builder;
pub mod page;
pub mod reader;

#[cfg(test)]
#[path = "../paged_index_tests.rs"]
mod tests;

/// Entry and byte budgets of one commit-index page member.
pub const COMMIT_PAGE_ENTRIES: u32 = 128;
pub const COMMIT_PAGE_BYTES: usize = 64 * 1024;

/// One page's fixed-size descriptor: its position range, the exact commit and
/// version id ranges of its rows, and its content address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitIndexPageRef {
    /// Patch position of the page's first row.
    pub first: u64,
    pub entries: u32,
    /// Inclusive `[min, max]` commit ids of the page's rows.
    pub commits: [u64; 2],
    /// Inclusive `[min, max]` version ids of the page's rows.
    pub versions: [u64; 2],
    pub member: RelationalDurableMemberLocator,
}

impl CommitIndexPageRef {
    /// One past the page's last position.
    pub fn end(&self) -> Option<u64> {
        self.first.checked_add(u64::from(self.entries))
    }

    pub fn holds_position(&self, position: u64) -> bool {
        position >= self.first && self.end().is_some_and(|end| position < end)
    }
}

/// The commit index a `CHK24` manifest retains.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointCommitIndex {
    pub count: u64,
    /// The last entry, so coverage is checked without reading a page.
    pub tail: Option<RelationalCommitIndexEntry>,
    /// The entries of the checkpoint's selected roots, ascending by position.
    pub heads: Vec<RelationalCommitIndexEntry>,
    pub pages: Vec<CommitIndexPageRef>,
}

impl CheckpointCommitIndex {
    /// Entry count, saturating where `usize` is narrower than the count.
    pub fn len(&self) -> usize {
        usize::try_from(self.count).unwrap_or(usize::MAX)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Page members, in position order; checkpoint custody pins all of them.
    pub fn page_members(&self) -> impl Iterator<Item = RelationalDurableMemberLocator> + '_ {
        self.pages.iter().map(|page| page.member)
    }

    pub fn head(&self, commit: CommitId) -> Option<&RelationalCommitIndexEntry> {
        self.heads
            .iter()
            .find(|entry| entry.commit.commit_id == commit)
    }

    /// The inline entry at `position` (the tail or a head), if any.
    pub fn inline_at(&self, position: u64) -> Option<&RelationalCommitIndexEntry> {
        if let Some(tail) = self.tail.as_ref().filter(|tail| tail.position.0 == position) {
            return Some(tail);
        }
        self.heads
            .binary_search_by_key(&position, |entry| entry.position.0)
            .ok()
            .map(|index| &self.heads[index])
    }

    /// The page holding `position`, by binary search over page ranges.
    pub fn page_for(&self, position: PatchStreamPosition) -> Option<usize> {
        let index = self
            .pages
            .partition_point(|page| page.first <= position.0)
            .checked_sub(1)?;
        self.pages[index]
            .holds_position(position.0)
            .then_some(index)
    }

    /// Structural validation that reads no page: contiguous non-empty pages
    /// within budget, tight unique members, a count equal to the page sum, and
    /// inline entries that lie inside their page's ranges.
    pub fn validate(&self) -> Result<(), &'static str> {
        let mut next = 1_u64;
        let mut members = Vec::with_capacity(self.pages.len());
        for page in &self.pages {
            if page.first != next
                || page.entries == 0
                || page.entries > COMMIT_PAGE_ENTRIES
                || page.commits[0] == 0
                || page.commits[0] > page.commits[1]
                || page.versions[0] > page.versions[1]
                || page.member.length == 0
                || page.member.length > COMMIT_PAGE_BYTES as u64
            {
                return Err("commit index page descriptor is invalid or not contiguous");
            }
            next = page.end().ok_or("commit index page range overflows")?;
            members.push(page.member);
        }
        if next - 1 != self.count {
            return Err("commit index count disagrees with its pages");
        }
        members.sort_unstable();
        if members.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("commit index pages repeat a member");
        }
        match &self.tail {
            None if self.count == 0 => {}
            Some(tail) if tail.position.0 == self.count => self.within_page(tail)?,
            _ => return Err("commit index tail is not its last entry"),
        }
        let mut prior: Option<&RelationalCommitIndexEntry> = None;
        for head in &self.heads {
            if prior.is_some_and(|prior| {
                prior.position >= head.position || prior.commit.commit_id == head.commit.commit_id
            }) {
                return Err("commit index heads are unordered or repeated");
            }
            self.within_page(head)?;
            prior = Some(head);
        }
        Ok(())
    }

    fn within_page(&self, entry: &RelationalCommitIndexEntry) -> Result<(), &'static str> {
        let page = self
            .page_for(entry.position)
            .map(|index| &self.pages[index])
            .ok_or("commit index inline entry has no page")?;
        let (commit, version) = (entry.commit.commit_id.0, entry.commit.version_id.0);
        if commit < page.commits[0]
            || commit > page.commits[1]
            || version < page.versions[0]
            || version > page.versions[1]
        {
            return Err("commit index inline entry lies outside its page ranges");
        }
        Ok(())
    }
}
