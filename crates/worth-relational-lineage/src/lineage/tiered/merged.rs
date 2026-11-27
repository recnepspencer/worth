//! Ordered scans over a tiered table and its encoded changes for capture.
//!
//! Scans are the only full-range reads of a tiered table: audits, history
//! selection over one scheme's groups, retention compaction and capture.
//! Point reads never scan.
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::{RowRef, TieredMap};
use crate::lineage::naming_pages::codec::NamingCodec;
use crate::lineage::naming_pages::store::ColdNamingStore;

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
        let mut rows = Vec::new();
        self.scan_merged_with(&[self.table()], store, |key, value| {
            rows.push((self.row_key(&key), Some(value.to_bytes())));
            Ok(true)
        })?;
        Ok(rows)
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
