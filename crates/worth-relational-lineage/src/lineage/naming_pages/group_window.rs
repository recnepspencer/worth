//! Per-branch commit-window index over primary `GROUP_LOG` pages so cold
//! merge preparation routes by `(branch_digest, commit)` before opening any
//! `GROUP_LOG` page payload.

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
    /// Sorted ascending by `(last_commit, first_commit, page_idx)`.
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
            for span in &page.group_log_branches {
                raw.entry(span.branch_digest)
                    .or_default()
                    .push(GroupLogEntry {
                        page_idx,
                        first_commit: span.first_commit,
                        last_commit: span.last_commit,
                    });
            }
        }
        let mut by_branch = BTreeMap::new();
        for (digest, mut entries) in raw {
            entries.sort_by_key(|e| (e.last_commit, e.first_commit, e.page_idx));
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

/// Because `best_limit` only ever decreases as branches iterate in ascending
/// digest order, any earlier branch `span.branch_digest < current` whose span
/// still qualifies under `best_limit` is guaranteed to have already scanned
/// `page`.
fn already_scanned_by_earlier_branch(
    page: &NamingPageRef,
    current: &[u8; 32],
    digest_bounds: &BTreeMap<[u8; 32], CommitId>,
    after: CommitId,
    best_limit: u64,
) -> bool {
    page.group_log_branches.iter().any(|span| {
        &span.branch_digest < current
            && digest_bounds
                .get(&span.branch_digest)
                .is_some_and(|&bound| {
                    let effective = bound.0.min(best_limit);
                    span.last_commit > after.0 && span.first_commit <= effective
                })
    })
}

impl ColdNamingStore {
    /// Streams primary `GROUP_LOG` pages overlapping `(after, digest_bounds[b]]`
    /// per branch with `O(log P_b)` pre-base seek, `O(1)` post-bound
    /// termination, `O(1)` deduplication scratch space, and key filtering
    /// before chunk reassembly.
    ///
    /// `visit(at, frame)` returns `Ok(true)` when it accepts a match at `at`
    /// (tightening the global search ceiling to `< at` and stopping the current
    /// page immediately), or `Ok(false)` to continue scanning.
    pub fn scan_group_log_after(
        &self,
        digest_bounds: &BTreeMap<[u8; 32], CommitId>,
        after: CommitId,
        mut visit: impl FnMut(CommitId, &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let group_log = &self.state().group_log;
        let mut best_limit = u64::MAX;
        for (&digest, &bound) in digest_bounds {
            if bound <= after {
                continue;
            }
            let Some(window) = self.group_log_windows.by_branch.get(&digest) else {
                continue;
            };
            let start = window.entries.partition_point(|e| e.last_commit <= after.0);
            for idx in start..window.entries.len() {
                let effective_bound = bound.0.min(best_limit);
                if effective_bound <= after.0 || window.suffix_min_first[idx] > effective_bound {
                    break;
                }
                let entry = &window.entries[idx];
                if entry.first_commit > effective_bound {
                    continue;
                }
                let page_ref = &group_log[entry.page_idx];
                if already_scanned_by_earlier_branch(
                    page_ref,
                    &digest,
                    digest_bounds,
                    after,
                    best_limit,
                ) {
                    continue;
                }
                let page_limit = best_limit;
                let _ = self.scan_page_filtered(
                    group_log,
                    page_ref,
                    &[GROUP_LOG],
                    |key| {
                        let (at, owner_digest) = parse_group_log_key(key)?;
                        Ok(at > after
                            && at.0 <= page_limit
                            && digest_bounds.get(&owner_digest).is_some_and(|b| at <= *b))
                    },
                    |key, frame| {
                        let (at, _) = parse_group_log_key(key)?;
                        if at.0 > best_limit {
                            return Ok(false);
                        }
                        if visit(at, frame)? {
                            best_limit = best_limit.min(at.0.saturating_sub(1));
                            return Ok(false);
                        }
                        Ok(true)
                    },
                )?;
            }
        }
        Ok(())
    }
}
