//! Bounded diff counting and encoding for [`FrozenBranch`] delta sealing.

use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::builder::PageChange;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::{self, table_domain};
use crate::lineage::naming_pages::BranchNamingRoot;
use crate::lineage::tiered::NamingBaseLink;

use super::super::name_indexes::BranchNameIndex;
use super::FrozenBranch;

/// Checks whether a delta root with `has_own_domain` satisfies
/// `manifest_validation/naming/delta.rs` alias header authority against `base`.
pub(super) fn alias_headers_compatible(
    root: &BranchNamingRoot,
    base: &BranchNamingRoot,
    has_own_domain: impl Fn(u8) -> bool,
) -> bool {
    if root.has_active_domain(1)
        && !has_own_domain(1)
        && (!base.has_active_domain(1) || base.naming != root.naming)
    {
        return false;
    }
    if root.has_active_domain(2) && base.has_active_domain(2) {
        let covers_retention = base
            .index
            .zip(root.index)
            .is_some_and(|(b, r)| b.retention_floor <= r.retention_floor);
        if !covers_retention {
            return false;
        }
    }
    if root.has_active_domain(2) && !has_own_domain(2) {
        let same_earliest = base
            .index
            .zip(root.index)
            .is_some_and(|(b, r)| b.earliest_commit == r.earliest_commit);
        let head_mismatch_with_event_diff = base.naming != root.naming
            && base
                .index
                .zip(root.index)
                .is_some_and(|(b, r)| b.head != r.head);
        if !base.has_active_domain(2) || !same_earliest || head_mismatch_with_event_diff {
            return false;
        }
    }
    if root.has_active_domain(tables::GROUP_STATE_DOMAIN)
        && !has_own_domain(tables::GROUP_STATE_DOMAIN)
    {
        let same_groups = base
            .groups
            .as_ref()
            .zip(root.groups.as_ref())
            .is_some_and(|(b, r)| b.schemes == r.schemes && b.earliest_commit == r.earliest_commit);
        if !base.has_active_domain(tables::GROUP_STATE_DOMAIN) || !same_groups {
            return false;
        }
    }
    true
}

pub(super) fn ops_have_domain<T>(ops: &[(Vec<u8>, T)], domain: u8) -> bool {
    ops.iter()
        .any(|(key, _)| key.first().copied().and_then(table_domain) == Some(domain))
}

pub(super) fn shared_cold_for<'a>(
    cold: Option<&'a ColdNamingStore>,
    b_base: Option<&NamingBaseLink>,
    c_base: Option<&NamingBaseLink>,
) -> Option<Option<(&'a ColdNamingStore, &'a BranchNamingRoot)>> {
    match (b_base, c_base) {
        (None, None) => Some(None),
        (Some(lb), Some(lc)) => {
            let store = cold?;
            let (rb, rc) = (lb.root_in(store)?, lc.root_in(store)?);
            (rb.branch == rc.branch).then_some(Some((store, rb)))
        }
        _ => None,
    }
}

impl FrozenBranch {
    /// Estimates the emitted delta row count against `base` without allocating,
    /// encoding keys/values, or faulting cold pages, aborting early above `budget`.
    pub(super) fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize> {
        let mut total = 0usize;
        if let Some(Ok(a)) = &self.naming {
            let rem = budget.saturating_sub(total);
            let part = if let Some(Ok(b)) = &base.naming {
                a.count_diff_against_bounded(b, has_cold_base, rem, visited)?
            } else if a.base().is_none() {
                let len = a.delta_len();
                if len > rem {
                    return None;
                }
                len
            } else {
                return None;
            };
            total += part;
            if total > budget {
                return None;
            }
        }
        if let Some(BranchNameIndex::Current(a)) = &self.index {
            let rem = budget.saturating_sub(total);
            let part = match &base.index {
                Some(BranchNameIndex::Current(b)) => {
                    a.count_diff_against_bounded(b, has_cold_base, rem, visited)?
                }
                _ if a.base().is_none() => {
                    let len = a.delta_len();
                    if len > rem {
                        return None;
                    }
                    len
                }
                _ => return None,
            };
            total += part;
            if total > budget {
                return None;
            }
        }
        if let Some(Ok(a)) = &self.groups {
            let rem = budget.saturating_sub(total);
            let part = match &base.groups {
                Some(Ok(b)) => a.count_diff_against_bounded(b, has_cold_base, rem, visited)?,
                _ if a.base().is_none() => {
                    let len = a.delta_len();
                    if len > rem {
                        return None;
                    }
                    len
                }
                _ => return None,
            };
            total += part;
            if total > budget {
                return None;
            }
        }
        Some(total)
    }

    pub(super) fn diff_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(&ColdNamingStore, &BranchNamingRoot)>,
        budget: usize,
    ) -> Result<Option<Vec<(Vec<u8>, PageChange)>>, Cause> {
        let mut out = Vec::new();
        if let Some(Ok(a)) = &self.naming {
            let rem = budget.saturating_sub(out.len());
            if let Some(Ok(b)) = &base.naming {
                let Some(part) = a.diff_encoded_against_bounded(b, cold_base, rem)? else {
                    return Ok(None);
                };
                out.extend(part);
            } else {
                if a.base().is_some() {
                    return Ok(None);
                }
                let changes = a.encoded_changes();
                if changes.len() > rem {
                    return Ok(None);
                }
                out.extend(
                    changes
                        .into_iter()
                        .filter_map(|(k, v)| Some((k, PageChange::Put(v?)))),
                );
            }
        }
        if let Some(BranchNameIndex::Current(a)) = &self.index {
            let rem = budget.saturating_sub(out.len());
            let part = match &base.index {
                Some(BranchNameIndex::Current(b)) => {
                    a.diff_encoded_against_bounded(b, cold_base, rem)?
                }
                _ if a.base().is_none() => a.encoded_puts_bounded(rem),
                _ => None,
            };
            let Some(part) = part else { return Ok(None) };
            out.extend(part);
        }
        if let Some(Ok(a)) = &self.groups {
            let rem = budget.saturating_sub(out.len());
            let part = match &base.groups {
                Some(Ok(b)) => a.diff_encoded_against_bounded(b, cold_base, rem)?,
                _ if a.base().is_none() => a.encoded_puts_bounded(rem),
                _ => None,
            };
            let Some(part) = part else { return Ok(None) };
            out.extend(part);
        }
        Ok((out.len() <= budget).then_some(out))
    }
}
