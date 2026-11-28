//! Ordered scans over a tiered table and its encoded changes for capture.
//!
//! Scans are the only full-range reads of a tiered table: audits, history
//! selection over one scheme's groups, retention compaction and capture.
//! Point reads never scan.
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::cow_map::CowDiff;
use super::{NamingBaseLink, RowRef, TieredMap};
use crate::lineage::naming_pages::builder::PageChange;
use crate::lineage::naming_pages::codec::NamingCodec;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::BranchNamingRoot;

impl NamingBaseLink {
    /// The branch id whose `BranchNamingRoot` this link resolves to in `store`.
    pub fn resolved_branch_in<'a>(
        &self,
        store: &'a ColdNamingStore,
    ) -> Option<&'a worth_relational_contracts::internal_port::history::data::coordinates::BranchId>
    {
        self.root_in(store).map(|root| &root.branch)
    }
}

impl<K, V> TieredMap<K, V>
where
    K: NamingCodec + Ord + Clone,
    V: NamingCodec,
{
    fn scan_merged_with(
        &self,
        prefix: &[u8],
        pinned_store: Option<&ColdNamingStore>,
        mut visit: impl FnMut(K, RowRef<'_, V>) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let mut delta = self
            .delta
            .iter()
            .map(|(key, value)| (self.row_key(key), key, value))
            .filter(|(row, _, _)| row.starts_with(prefix))
            .collect::<Vec<_>>();
        delta.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let mut pending = delta.into_iter().peekable();
        let mut stopped = false;
        if let Some(base) = self.base() {
            let mut visit_base = |row: &[u8], bytes: &[u8]| -> Result<bool, Cause> {
                while let Some((key_bytes, key, value)) = pending.peek() {
                    if key_bytes.as_slice() >= row {
                        break;
                    }
                    if let Some(value) = value.as_ref()
                        && !visit((*key).clone(), RowRef::Borrowed(value))?
                    {
                        stopped = true;
                        return Ok(false);
                    }
                    pending.next();
                }
                if pending
                    .peek()
                    .is_some_and(|(key_bytes, _, _)| key_bytes.as_slice() == row)
                {
                    let (_, key, value) = pending.next().ok_or(Cause::Damaged)?;
                    if let Some(value) = value.as_ref()
                        && !visit(key.clone(), RowRef::Borrowed(value))?
                    {
                        stopped = true;
                        return Ok(false);
                    }
                    return Ok(true);
                }
                let key = K::from_bytes(row.get(1..).ok_or(Cause::Damaged)?)?;
                let value = V::from_bytes(bytes)?;
                let keep = visit(key, RowRef::Owned(value))?;
                stopped = !keep;
                Ok(keep)
            };
            match pinned_store {
                Some(store) => base.scan_in(store, prefix, &mut visit_base)?,
                None => base.scan(prefix, &mut visit_base)?,
            }
        }
        if stopped {
            return Ok(());
        }
        for (_, key, value) in pending {
            if let Some(value) = value.as_ref()
                && !visit(key.clone(), RowRef::Borrowed(value))?
            {
                break;
            }
        }
        Ok(())
    }

    /// Visits every live row whose row key starts with `prefix` (which
    /// begins with this table's tag), ascending by row key; `visit` returns
    /// `false` to stop.
    ///
    /// Complexity: O(delta log delta + rows in the prefix range), faulting
    /// only the base pages that range spans.
    pub fn scan_merged(
        &self,
        prefix: &[u8],
        visit: impl FnMut(K, RowRef<'_, V>) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        self.scan_merged_with(prefix, None, visit)
    }

    /// Visits every live row whose row key starts with `prefix`, ascending,
    /// seeking the delta from `lower` (the least key whose row key carries
    /// `prefix`) rather than walking it; `visit` returns `false` to stop.
    /// Keys sharing `prefix` must order exactly as their row keys do.
    ///
    /// Complexity: O(log N + visited rows), faulting only the base pages the
    /// visited range spans.
    pub fn scan_prefix_from(
        &self,
        lower: &K,
        prefix: &[u8],
        mut visit: impl FnMut(K, RowRef<'_, V>) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let mut pending = self
            .delta
            .iter_from(lower)
            .map(|(key, value)| (self.row_key(key), key, value))
            .take_while(|(row, _, _)| row.starts_with(prefix))
            .peekable();
        let mut stopped = false;
        if let Some(base) = self.base() {
            base.scan(prefix, |row, bytes| {
                while let Some((key_bytes, key, value)) = pending.peek() {
                    if key_bytes.as_slice() > row {
                        break;
                    }
                    let shadows = key_bytes.as_slice() == row;
                    if let Some(value) = value.as_ref()
                        && !visit((*key).clone(), RowRef::Borrowed(value))?
                    {
                        stopped = true;
                        return Ok(false);
                    }
                    pending.next();
                    if shadows {
                        return Ok(true);
                    }
                }
                let key = K::from_bytes(row.get(1..).ok_or(Cause::Damaged)?)?;
                let keep = visit(key, RowRef::Owned(V::from_bytes(bytes)?))?;
                stopped = !keep;
                Ok(keep)
            })?;
        }
        if stopped {
            return Ok(());
        }
        for (_, key, value) in pending {
            if let Some(value) = value.as_ref()
                && !visit(key.clone(), RowRef::Borrowed(value))?
            {
                break;
            }
        }
        Ok(())
    }

    /// Every live row of the table, ascending by row key, duplicating borrowed
    /// resident rows through `duplicate`.
    pub fn collect_merged_with(
        &self,
        mut duplicate: impl FnMut(&V) -> V,
    ) -> Result<Vec<(K, V)>, Cause> {
        let mut rows = Vec::new();
        self.scan_merged(&[self.table()], |key, value| {
            rows.push((key, value.into_owned_with(&mut duplicate)));
            Ok(true)
        })?;
        Ok(rows)
    }

    /// Every live row of this table merged against a specific pinned store,
    /// encoded for mixed-base capture (e.g. cold historical fork).
    pub fn collect_encoded_in(
        &self,
        store: Option<&ColdNamingStore>,
    ) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>, Cause> {
        self.collect_encoded_prefix_in(&[self.table()], store)
    }

    /// Every live row of this table whose row key starts with `prefix`, merged
    /// against `store`, encoded for scheme-scoped mixed-base capture.
    pub fn collect_encoded_prefix_in(
        &self,
        prefix: &[u8],
        store: Option<&ColdNamingStore>,
    ) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>, Cause> {
        let mut rows = Vec::new();
        self.scan_merged_with(prefix, store, |key, value| {
            rows.push((self.row_key(&key), Some(value.to_bytes())));
            Ok(true)
        })?;
        Ok(rows)
    }

    /// Number of resident delta entries (`O(1)`).
    pub fn delta_len(&self) -> usize {
        self.delta.len()
    }

    /// Estimates the emitted delta row count against `base` without allocating,
    /// encoding keys/values, or faulting cold pages, aborting early above `budget`
    /// and adding every visited non-shared node (including aborted probes) to `visited`.
    pub fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize>
    where
        V: PartialEq,
    {
        let mut count = 0usize;
        let (completed, stats) = self
            .delta
            .visit_diff_against_with_stats(&base.delta, |diff| {
                let emits = match diff {
                    CowDiff::Set(_, Some(_), _) => true,
                    CowDiff::Set(_, None, Some(None)) => false,
                    CowDiff::Set(_, None, base_slot) => {
                        has_cold_base || matches!(base_slot, Some(Some(_)))
                    }
                    CowDiff::Removed(_, base_slot) => has_cold_base || base_slot.is_some(),
                };
                if emits {
                    count += 1;
                    if count > budget {
                        return false;
                    }
                }
                true
            });
        *visited += stats.visited;
        completed.then_some(count)
    }

    /// Structural-sharing diff from `base` (sharing the exact same cold base
    /// `cold_base`, or both pure-resident when `None`) to `self`, aborting
    /// early if emitted rows would exceed `budget`.
    ///
    /// Returns `Ok(Some(changes))` with `changes.len() <= budget`, or
    /// `Ok(None)` when the budget was exceeded.
    pub fn diff_encoded_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(&ColdNamingStore, &BranchNamingRoot)>,
        budget: usize,
    ) -> Result<Option<Vec<(Vec<u8>, PageChange)>>, Cause>
    where
        V: PartialEq,
    {
        let Some((diffs, _)) = self.delta.diff_against_bounded(&base.delta, budget) else {
            return Ok(None);
        };
        let mut out = Vec::with_capacity(diffs.len());
        for diff in diffs {
            match diff {
                CowDiff::Set(key, Some(val), _) => {
                    out.push((self.row_key(key), PageChange::Put(val.to_bytes())));
                }
                CowDiff::Set(key, None, base_slot) => {
                    // `self` tombstoned `key`. Emit a tombstone unless `base`
                    // also has a tombstone (or pure-resident `base` lacks `key`).
                    if matches!(base_slot, Some(None)) {
                        continue;
                    }
                    if cold_base.is_some() || matches!(base_slot, Some(Some(_))) {
                        out.push((self.row_key(key), PageChange::Tombstone));
                    }
                }
                CowDiff::Removed(key, base_slot) => {
                    let rk = self.row_key(key);
                    match cold_base {
                        None => {
                            if base_slot.is_some() {
                                out.push((rk, PageChange::Tombstone));
                            }
                        }
                        Some((store, cold_root)) => match store.get_root(cold_root, &rk)? {
                            Some(bytes) => out.push((rk, PageChange::Put(bytes))),
                            None => {
                                if base_slot.is_some() {
                                    out.push((rk, PageChange::Tombstone));
                                }
                            }
                        },
                    }
                }
            }
            if out.len() > budget {
                return Ok(None);
            }
        }
        Ok(Some(out))
    }

    /// Every resident live entry of this table as `PageChange::Tombstone`
    /// (used when a scheme exists on `base` but not on `self`).
    pub fn encoded_tombstones_bounded(&self, budget: usize) -> Option<Vec<(Vec<u8>, PageChange)>> {
        if self.delta.len() > budget {
            return None;
        }
        let mut out = Vec::new();
        for (key, value) in self.delta.iter() {
            if value.is_some() {
                if out.len() >= budget {
                    return None;
                }
                out.push((self.row_key(key), PageChange::Tombstone));
            }
        }
        Some(out)
    }

    /// Every resident live entry of this table as `PageChange::Put`.
    pub fn encoded_puts_bounded(&self, budget: usize) -> Option<Vec<(Vec<u8>, PageChange)>> {
        if self.delta.len() > budget {
            return None;
        }
        let mut out = Vec::new();
        for (key, value) in self.delta.iter() {
            if let Some(val) = value.as_ref() {
                if out.len() >= budget {
                    return None;
                }
                out.push((self.row_key(key), PageChange::Put(val.to_bytes())));
            }
        }
        Some(out)
    }

    /// The delta as encoded row changes for capture, ascending by row key.
    ///
    /// Complexity: O(delta log delta).
    pub fn encoded_changes(&self) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
        if !self.has_delta() {
            return Vec::new();
        }
        let mut changes = self
            .delta
            .iter()
            .map(|(key, value)| (self.row_key(key), value.as_ref().map(V::to_bytes)))
            .collect::<Vec<_>>();
        changes.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        changes
    }
}

impl<K, V> TieredMap<K, V>
where
    K: NamingCodec + Ord + Clone,
    V: NamingCodec + Clone,
{
    /// Every live row of the table, ascending by row key.
    pub fn collect_merged(&self) -> Result<Vec<(K, V)>, Cause> {
        self.collect_merged_with(Clone::clone)
    }
}
