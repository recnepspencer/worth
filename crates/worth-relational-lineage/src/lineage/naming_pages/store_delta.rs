//! Delta-base chain resolution (`get_root` and `scan_root`) for `ColdNamingStore`.
//!
//! Resolves lookups and ordered prefix scans across a `BranchNamingRoot`'s
//! `delta_base` chain (`O(depth * log pages)` for point lookups, streaming
//! k-way merge for prefix scans). Tombstones (`VALUE_TOMBSTONE_TAG`) shadow
//! inherited base rows cleanly and are rejected as `Cause::Damaged` if ever
//! encountered in a non-delta root.

use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::store::ColdNamingStore;
use super::tables::{parse_value_envelope, table_domain, ValueEnvelope};
use super::{BranchNamingRoot, NamingPageRef, MAX_DELTA_DEPTH};

struct LayerCursor<'a> {
    pages: &'a [NamingPageRef],
    is_delta: bool,
    next_page: usize,
    rows: std::vec::IntoIter<(Vec<u8>, Vec<u8>)>,
    current: Option<(Vec<u8>, Vec<u8>)>,
}

impl<'a> LayerCursor<'a> {
    fn open(
        store: &ColdNamingStore,
        pages: &'a [NamingPageRef],
        is_delta: bool,
        prefix: &[u8],
    ) -> Result<Self, Cause> {
        let start = pages.partition_point(|page| page.last.as_slice() < prefix);
        let mut cursor = Self {
            pages,
            is_delta,
            next_page: start,
            rows: Vec::new().into_iter(),
            current: None,
        };
        cursor.advance(store, prefix)?;
        Ok(cursor)
    }

    fn advance(&mut self, store: &ColdNamingStore, prefix: &[u8]) -> Result<(), Cause> {
        loop {
            if let Some(next) = self.rows.next() {
                self.current = Some(next);
                return Ok(());
            }
            let Some(reference) = self.pages.get(self.next_page) else {
                self.current = None;
                return Ok(());
            };
            if !reference.first.starts_with(prefix) && reference.first.as_slice() > prefix {
                self.current = None;
                return Ok(());
            }
            self.next_page += 1;
            let page = store.page(reference)?;
            let mut staged = Vec::new();
            for (key, raw) in &page.page.rows {
                if key.as_slice() < prefix {
                    continue;
                }
                if !key.starts_with(prefix) {
                    self.next_page = self.pages.len();
                    break;
                }
                staged.push((key.clone(), raw.clone()));
            }
            // Drop the cached page pin before advancing to any other layer or chunk.
            drop(page);
            self.rows = staged.into_iter();
        }
    }
}

fn has_overlapping_page(pages: &[NamingPageRef], prefix: &[u8]) -> bool {
    let slot = pages.partition_point(|page| page.last.as_slice() < prefix);
    pages
        .get(slot)
        .is_some_and(|p| p.first.starts_with(prefix) || p.first.as_slice() <= prefix)
}

impl ColdNamingStore {
    fn active_chain<'a>(
        &'a self,
        root: &'a BranchNamingRoot,
        domain: u8,
    ) -> Result<Vec<&'a BranchNamingRoot>, Cause> {
        let mut chain = vec![root];
        let mut current = root;
        while let Some(base_branch) = &current.delta_base {
            if !current.has_active_domain(domain) {
                break;
            }
            if chain.len() > MAX_DELTA_DEPTH {
                return Err(Cause::Damaged);
            }
            let next = self.state().root(base_branch).ok_or(Cause::Damaged)?;
            if !next.has_active_domain(domain) {
                break;
            }
            chain.push(next);
            current = next;
        }
        Ok(chain)
    }

    /// The value of row `key` in `root`, following `delta_base` when absent
    /// and stopping on a tombstone.
    ///
    /// Complexity: `O(depth * (log pages + log rows))` for inline values.
    pub fn get_root(&self, root: &BranchNamingRoot, key: &[u8]) -> Result<Option<Vec<u8>>, Cause> {
        if root.delta_base.is_none() {
            return self.get(&root.pages, key);
        }
        let domain = key
            .first()
            .copied()
            .and_then(table_domain)
            .ok_or(Cause::Damaged)?;
        for layer in self.active_chain(root, domain)? {
            if let Some(raw) = self.raw_get(&layer.pages, key)? {
                return match parse_value_envelope(&raw)? {
                    ValueEnvelope::Tombstone => {
                        if layer.delta_base.is_none() {
                            Err(Cause::Damaged)
                        } else {
                            Ok(None)
                        }
                    }
                    ValueEnvelope::Inline => Ok(Some(raw[1..].to_vec())),
                    ValueEnvelope::Chunked { .. } => {
                        Ok(Some(self.reassemble(&layer.pages, key, &raw)?))
                    }
                };
            }
        }
        Ok(None)
    }

    /// Visits every live row of `root` starting with `prefix`, ascending,
    /// resolving `delta_base` layers with tombstones shadowing inherited rows.
    pub fn scan_root(
        &self,
        root: &BranchNamingRoot,
        prefix: &[u8],
        mut visit: impl FnMut(&[u8], &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        if root.delta_base.is_none() {
            return self.scan(&root.pages, prefix, visit);
        }
        let domain = prefix
            .first()
            .copied()
            .and_then(table_domain)
            .ok_or(Cause::Damaged)?;
        let chain = self.active_chain(root, domain)?;
        let active: Vec<&BranchNamingRoot> = chain
            .into_iter()
            .filter(|layer| has_overlapping_page(&layer.pages, prefix))
            .collect();
        if active.is_empty() {
            return Ok(());
        }
        if active.len() == 1 && active[0].delta_base.is_none() {
            return self.scan(&active[0].pages, prefix, visit);
        }
        let mut cursors = Vec::with_capacity(active.len());
        for layer in active {
            cursors.push(LayerCursor::open(
                self,
                &layer.pages,
                layer.delta_base.is_some(),
                prefix,
            )?);
        }
        loop {
            let mut winner: Option<usize> = None;
            for (idx, cursor) in cursors.iter().enumerate() {
                let Some((key, _)) = &cursor.current else {
                    continue;
                };
                let better = match winner {
                    None => true,
                    Some(w) => key < &cursors[w].current.as_ref().unwrap().0,
                };
                if better {
                    winner = Some(idx);
                }
            }
            let Some(win_idx) = winner else {
                return Ok(());
            };
            let (win_key, win_raw) = cursors[win_idx].current.take().unwrap();
            let win_is_delta = cursors[win_idx].is_delta;
            let win_pages = cursors[win_idx].pages;
            let keep = match parse_value_envelope(&win_raw)? {
                ValueEnvelope::Tombstone => {
                    if !win_is_delta {
                        return Err(Cause::Damaged);
                    }
                    true
                }
                ValueEnvelope::Inline => visit(&win_key, &win_raw[1..])?,
                ValueEnvelope::Chunked { .. } => {
                    let full = self.reassemble(win_pages, &win_key, &win_raw)?;
                    visit(&win_key, &full)?
                }
            };
            if !keep {
                return Ok(());
            }
            cursors[win_idx].advance(self, prefix)?;
            for cursor in &mut cursors[win_idx + 1..] {
                if cursor
                    .current
                    .as_ref()
                    .is_some_and(|(key, _)| key.as_slice() == win_key.as_slice())
                {
                    cursor.current = None;
                    cursor.advance(self, prefix)?;
                }
            }
        }
    }
}
