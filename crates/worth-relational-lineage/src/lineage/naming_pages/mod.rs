//! `CHK24` paged persistent-naming state (`G7` for Persistent Naming).
//!
//! Every branch's derived naming basis, name index and group state live in
//! one sorted run of `(table key, value)` rows split into bounded
//! content-addressed pages. The manifest retains only [`CheckpointNamingState`]:
//! per branch, its resident scalar header and one [`NamingPageRef`] per page
//! (exact first and last key, row count, member). Validation reads no page;
//! a lookup binary-searches the descriptors and faults the one page whose key
//! range holds the key. Branches whose state is identical share pages by
//! content address. The published group lineage log is paged the same way,
//! keyed by commit, for the exact history selections replay.
use serde::{Deserialize, Serialize};

use crate::contracts::history::data::coordinates::BranchId;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;

pub mod builder;
pub mod codec;
mod codec_im;
mod group_window;
pub mod page;
pub mod store;
pub mod tables;
#[cfg(test)]
mod tests;

/// Row and byte budgets of one naming page member.
pub const NAMING_PAGE_ENTRIES: u32 = 256;
pub const NAMING_PAGE_BYTES: usize = 64 * 1024;
/// Bound on one physical table key (including continuation chunk keys).
pub const NAMING_KEY_BYTES: usize = 256;
/// Bound on one logical (non-chunk) row key.
pub const NAMING_LOGICAL_KEY_BYTES: usize = 192;
/// Bound on one installed group scheme id in a branch header.
pub const SCHEME_ID_BYTES: usize = 128;

/// Per-branch publication-commit span carried on a primary `GROUP_LOG` page
/// descriptor so cold merge preparation routes by `(branch, commit)` before
/// opening any page payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupLogBranchSpan {
    pub branch_digest: [u8; 32],
    pub first_commit: u64,
    pub last_commit: u64,
}

/// One page's fixed-bound descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingPageRef {
    /// The page's first and last row keys (inclusive, table tag first).
    pub first: Vec<u8>,
    pub last: Vec<u8>,
    pub entries: u32,
    #[serde(default)]
    pub continuation_chunks: u32,
    #[serde(default)]
    pub continuation_digest: Option<[u8; 32]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_log_branches: Vec<GroupLogBranchSpan>,
    pub member: RelationalDurableMemberLocator,
}

/// The derived naming basis header of one branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingStateHeader {
    pub earliest_event_commit: Option<u64>,
    pub latest_event_commit: Option<u64>,
    pub unavailable: bool,
}

/// The derived name index header of one branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameIndexHeader {
    pub earliest_commit: Option<u64>,
    pub head: Option<u64>,
    pub retention_floor: Option<u64>,
    pub unavailable: bool,
}

/// One installed group scheme a branch's group state holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeHeader {
    pub id: String,
    pub rule: u8,
}

/// The group state header of one branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupStateHeader {
    pub earliest_commit: Option<u64>,
    pub latest_commit: Option<u64>,
    pub schemes: Vec<SchemeHeader>,
    pub unavailable: bool,
}

/// One branch's resident header and page run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchNamingRoot {
    pub branch: BranchId,
    pub naming: Option<NamingStateHeader>,
    pub index: Option<NameIndexHeader>,
    pub groups: Option<GroupStateHeader>,
    pub pages: Vec<NamingPageRef>,
}

/// The naming state a `CHK24` manifest retains.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointNamingState {
    /// Strictly ascending by branch.
    pub branches: Vec<BranchNamingRoot>,
    /// Published group lineage, keyed `(commit, branch)`.
    pub group_log: Vec<NamingPageRef>,
    /// The newest commit the group log covers.
    pub group_log_through: Option<u64>,
}

impl CheckpointNamingState {
    /// The root `branch` holds, by binary search.
    ///
    /// Complexity: O(log branches).
    pub fn root(&self, branch: &BranchId) -> Option<&BranchNamingRoot> {
        self.branches
            .binary_search_by(|root| root.branch.cmp(branch))
            .ok()
            .map(|slot| &self.branches[slot])
    }

    /// Every page member, deduplicated; checkpoint custody pins all of them.
    pub fn page_members(&self) -> Vec<RelationalDurableMemberLocator> {
        let mut members = self
            .branches
            .iter()
            .flat_map(|root| &root.pages)
            .chain(&self.group_log)
            .map(|page| page.member)
            .collect::<Vec<_>>();
        members.sort_unstable();
        members.dedup();
        members
    }

    /// Structural validation that reads no page.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self
            .branches
            .windows(2)
            .any(|pair| pair[0].branch >= pair[1].branch)
        {
            return Err("naming roots are not strictly ascending by branch");
        }
        for root in &self.branches {
            validate_pages(&root.pages)?;
            if let Some(groups) = &root.groups {
                let mut prior: Option<&str> = None;
                for scheme in &groups.schemes {
                    if scheme.id.is_empty()
                        || scheme.id.len() > SCHEME_ID_BYTES
                        || !(1..=3).contains(&scheme.rule)
                        || prior.is_some_and(|prior| prior >= scheme.id.as_str())
                    {
                        return Err("naming root group schemes are invalid or unordered");
                    }
                    prior = Some(&scheme.id);
                }
            }
            if root
                .pages
                .iter()
                .any(|page| page.first[0] == tables::GROUP_LOG)
            {
                return Err("a branch root carries group log rows");
            }
        }
        validate_pages(&self.group_log)?;
        if self.group_log.iter().any(|page| {
            !matches!(page.first[0], tables::GROUP_LOG | tables::VALUE_CHUNK)
                || !matches!(page.last[0], tables::GROUP_LOG | tables::VALUE_CHUNK)
        }) {
            return Err("the group log carries foreign rows");
        }
        if !self.group_log.is_empty() && self.group_log_through.is_none() {
            return Err("the group log has no covered commit");
        }
        Ok(())
    }
}

/// Non-empty pages within budget whose key ranges ascend without overlap.
pub fn validate_pages(pages: &[NamingPageRef]) -> Result<(), &'static str> {
    let (mut prior, mut expected_chunks, mut actual_chunks): (Option<&NamingPageRef>, u64, u64) =
        (None, 0, 0);
    for page in pages {
        let key_ok = |key: &[u8]| !key.is_empty() && key.len() <= NAMING_KEY_BYTES;
        if page.entries == 0
            || page.entries > NAMING_PAGE_ENTRIES
            || !key_ok(&page.first)
            || !key_ok(&page.last)
            || page.first > page.last
            || (page.entries == 1) != (page.first == page.last)
            || tables::table_domain(page.first[0]).is_none()
            || tables::table_domain(page.first[0]) != tables::table_domain(page.last[0])
            || page.member.length == 0
            || page.member.length > NAMING_PAGE_BYTES as u64
            || prior.is_some_and(|prior| prior.last >= page.first)
        {
            return Err("naming page descriptor is invalid or out of order");
        }
        if page.first[0] == tables::VALUE_CHUNK {
            if page.continuation_chunks != 0
                || page.continuation_digest.is_some()
                || !page.group_log_branches.is_empty()
            {
                return Err("continuation chunk page carries nested continuations");
            }
            actual_chunks += u64::from(page.entries);
        } else {
            if (page.continuation_chunks == 0) != page.continuation_digest.is_none() {
                return Err("naming page continuation count and digest disagree");
            }
            if page.first[0] == tables::GROUP_LOG {
                let (first_commit, _) = tables::parse_group_log_key(&page.first)
                    .map_err(|_| "group log page first key is malformed")?;
                let (last_commit, _) = tables::parse_group_log_key(&page.last)
                    .map_err(|_| "group log page last key is malformed")?;
                if page.group_log_branches.is_empty()
                    || page.group_log_branches.len() > page.entries as usize
                    || page
                        .group_log_branches
                        .windows(2)
                        .any(|pair| pair[0].branch_digest >= pair[1].branch_digest)
                    || page.group_log_branches.iter().any(|span| {
                        span.first_commit == 0
                            || span.first_commit > span.last_commit
                            || span.first_commit < first_commit.0
                            || span.last_commit > last_commit.0
                    })
                {
                    return Err("group log page branch spans are invalid or out of bounds");
                }
            } else if !page.group_log_branches.is_empty() {
                return Err("non-group-log naming page carries group log branch spans");
            }
            expected_chunks += u64::from(page.continuation_chunks);
        }
        prior = Some(page);
    }
    if expected_chunks != actual_chunks {
        return Err("naming page continuation chunk count disagrees with chunk roster");
    }
    Ok(())
}

/// The page whose key range may hold `key`: the last page starting at or
/// before it, when `key` is not past its last row.
///
/// Complexity: O(log pages).
pub fn page_for(pages: &[NamingPageRef], key: &[u8]) -> Option<usize> {
    let slot = pages
        .partition_point(|page| page.first.as_slice() <= key)
        .checked_sub(1)?;
    (key <= pages[slot].last.as_slice()).then_some(slot)
}
