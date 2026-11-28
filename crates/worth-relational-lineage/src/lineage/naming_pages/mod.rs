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
pub(crate) mod tests;

/// Row and byte budgets of one naming page member.
pub const NAMING_PAGE_ENTRIES: u32 = 256;
pub const NAMING_PAGE_BYTES: usize = 64 * 1024;
/// Upper bound on ordinary physical keys, including their continuation keys.
pub const NAMING_KEY_BYTES: usize = 256;
/// Bound on one logical (non-chunk) row key.
pub const NAMING_LOGICAL_KEY_BYTES: usize = 192;
/// Qualified projected member rows fit alongside a maximum inline value (or
/// chunk) under the page writer's two-byte-per-input-byte bound.
const GROUP_MEMBER_LOGICAL_KEY_BYTES: usize = NAMING_PAGE_BYTES / 8 - 128;

pub(super) fn logical_key_limit(table: Option<u8>) -> usize {
    match table {
        Some(tables::GROUP_MEMBERS | tables::GROUP_MEMBER_OWNER) => GROUP_MEMBER_LOGICAL_KEY_BYTES,
        _ => NAMING_LOGICAL_KEY_BYTES,
    }
}

fn physical_key_fits(key: &[u8]) -> bool {
    match key.first().copied() {
        None => false,
        Some(tables::VALUE_CHUNK) => tables::parse_chunk_key(key)
            .is_some_and(|(base, _)| base.len() <= logical_key_limit(base.first().copied())),
        Some(table) => key.len() <= logical_key_limit(Some(table)),
    }
}
/// Bound on one installed group scheme id in a branch header.
pub const SCHEME_ID_BYTES: usize = 128;

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
        if page.entries == 0
            || page.entries > NAMING_PAGE_ENTRIES
            || !physical_key_fits(&page.first)
            || !physical_key_fits(&page.last)
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
            if page.continuation_chunks != 0 || page.continuation_digest.is_some() {
                return Err("continuation chunk page carries nested continuations");
            }
            actual_chunks += u64::from(page.entries);
        } else {
            if (page.continuation_chunks == 0) != page.continuation_digest.is_none() {
                return Err("naming page continuation count and digest disagree");
            }
            if page.first[0] == tables::GROUP_LOG {
                let (first_commit, first_digest) = tables::parse_group_log_key(&page.first)
                    .map_err(|_| "group log page first key is malformed")?;
                let (last_commit, last_digest) = tables::parse_group_log_key(&page.last)
                    .map_err(|_| "group log page last key is malformed")?;
                if first_commit.0 == 0 || first_commit > last_commit || first_digest != last_digest
                {
                    return Err(
                        "group log page endpoints must belong to one branch with ascending commits",
                    );
                }
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
