//! Per-branch commit-window index over single-branch primary `GROUP_LOG` pages
//! derived solely from validated page endpoint keys (`first`, `last`).
//!
//! Cold merge preparation performs a `k`-way merge over the source ancestry's
//! branch cursors (`O(|ancestry|)` scratch space) in strict ascending
//! checkpoint page order, stopping at the very first matching transition before
//! faulting any later page.

use std::collections::BTreeMap;

use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::store::ColdNamingStore;
use super::tables::{parse_group_log_key, GROUP_LOG};
use super::NamingPageRef;

#[derive(Debug)]
struct GroupLogEntry {
    page_idx: usize,
    first_commit: u64,
    last_commit: u64,
}

#[derive(Debug)]
struct GroupLogBranchWindow {
    /// Strictly ascending by `page_idx` (and `last_commit`, since single-branch
    /// `GROUP_LOG` pages ascend monotonically in `(ordinal, commit)`).
    entries: Vec<GroupLogEntry>,
    /// `suffix_min_first[i] = min_{j >= i} entries[j].first_commit`.
    suffix_min_first: Vec<u64>,
}

#[derive(Debug, Default)]
pub(super) struct GroupLogWindowIndex {
    by_branch: BTreeMap<[u8; 32], GroupLogBranchWindow>,
}

impl GroupLogWindowIndex {
    pub(super) fn build(group_log: &[NamingPageRef]) -> Self {
        let mut raw: BTreeMap<[u8; 32], Vec<GroupLogEntry>> = BTreeMap::new();
        for (page_idx, page) in group_log.iter().enumerate() {
            if page.first.first() != Some(&GROUP_LOG) {
                continue;
            }
            let Ok((first_commit, first_digest)) = parse_group_log_key(&page.first) else {
                continue;
            };
            let Ok((last_commit, last_digest)) = parse_group_log_key(&page.last) else {
                continue;
            };
            if first_digest != last_digest {
                continue;
            }
            raw.entry(first_digest).or_default().push(GroupLogEntry {
                page_idx,
                first_commit: first_commit.0,
                last_commit: last_commit.0,
            });
        }
        let mut by_branch = BTreeMap::new();
        for (digest, entries) in raw {
            let mut suffix_min_first = vec![0u64; entries.len()];
            let mut running = u64::MAX;
            for (idx, entry) in entries.iter().enumerate().rev() {
                running = running.min(entry.first_commit);
                suffix_min_first[idx] = running;
            }
            by_branch.insert(
                digest,
                GroupLogBranchWindow {
                    entries,
                    suffix_min_first,
                },
            );
        }
        Self { by_branch }
    }
}

struct BranchCursor<'a> {
    window: &'a GroupLogBranchWindow,
    bound: CommitId,
    pos: usize,
}

impl<'a> BranchCursor<'a> {
    fn new(window: &'a GroupLogBranchWindow, after: CommitId, bound: CommitId) -> Self {
        let pos = window.entries.partition_point(|e| e.last_commit <= after.0);
        let mut cursor = Self { window, bound, pos };
        cursor.skip_ineligible();
        cursor
    }

    fn skip_ineligible(&mut self) {
        while self.pos < self.window.entries.len() {
            if self.window.suffix_min_first[self.pos] > self.bound.0 {
                self.pos = self.window.entries.len();
                return;
            }
            if self.window.entries[self.pos].first_commit <= self.bound.0 {
                return;
            }
            self.pos += 1;
        }
    }

    fn peek_page_idx(&self) -> Option<usize> {
        self.window.entries.get(self.pos).map(|e| e.page_idx)
    }

    fn advance(&mut self) {
        if self.pos < self.window.entries.len() {
            self.pos += 1;
            self.skip_ineligible();
        }
    }
}

impl ColdNamingStore {
    /// Streams primary `GROUP_LOG` pages overlapping `(after, digest_bounds[b]]`
    /// in strict ascending checkpoint page order (`k`-way merge over per-branch
    /// cursors, `O(log P_b)` pre-base seek, `O(1)` post-bound stop, `O(|ancestry|)`
    /// scratch space), filtering keys before chunk reassembly and stopping
    /// immediately once `visit` accepts a transition.
    pub fn scan_group_log_after(
        &self,
        digest_bounds: &BTreeMap<[u8; 32], CommitId>,
        after: CommitId,
        mut visit: impl FnMut(CommitId, &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let group_log = &self.state().group_log;
        let mut cursors: Vec<BranchCursor<'_>> = Vec::with_capacity(digest_bounds.len());
        for (&digest, &bound) in digest_bounds {
            if bound <= after {
                continue;
            }
            if let Some(window) = self.group_log_windows.by_branch.get(&digest) {
                let cursor = BranchCursor::new(window, after, bound);
                if cursor.peek_page_idx().is_some() {
                    cursors.push(cursor);
                }
            }
        }
        while let Some((best_cursor_idx, page_idx)) = cursors
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.peek_page_idx().map(|idx| (i, idx)))
            .min_by_key(|&(_, page_idx)| page_idx)
        {
            cursors[best_cursor_idx].advance();
            let page_ref = &group_log[page_idx];
            let mut accepted = false;
            let _ = self.scan_page_filtered(
                group_log,
                page_ref,
                &[GROUP_LOG],
                |key| {
                    let (at, owner_digest) = parse_group_log_key(key)?;
                    Ok(at > after && digest_bounds.get(&owner_digest).is_some_and(|b| at <= *b))
                },
                |key, frame| {
                    let (at, _) = parse_group_log_key(key)?;
                    if visit(at, frame)? {
                        accepted = true;
                        return Ok(false);
                    }
                    Ok(true)
                },
            )?;
            if accepted {
                return Ok(());
            }
        }
        Ok(())
    }
}
