//! Lazy reader over a checkpoint's paged naming state.
//!
//! Opening reads nothing. Every page read goes through a source confined to
//! the state's own page roster and lands in a page cache enrolled in the
//! working set's historical pool, so lookups fault only the pages whose key
//! ranges they touch and resident pages evict under that pool's budget.
use std::sync::{Arc, OnceLock};

use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::{
    ColdMemberSource, ConfinedMemberSource,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSource as _;
use worth_relational_storage::internal_port::storage::residency::chunked::page_cache::{
    PageCacheObservation, PageEvictionGroup, ResidentPageCache,
};
use worth_relational_storage::internal_port::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant,
};

use super::page::ResidentNamingPage;
use super::tables::{chunk_key, parse_value_envelope, MAX_INLINE_VALUE_BYTES, VALUE_CHUNKED_TAG};
use super::{page_for, CheckpointNamingState, NamingPageRef, NAMING_PAGE_BYTES};

pub struct ColdNamingStore {
    state: Arc<CheckpointNamingState>,
    source: ConfinedMemberSource<ColdMemberSource>,
    group: Arc<PageEvictionGroup>,
    accountant: OnceLock<Arc<WorkingSetAccountant>>,
    cache: OnceLock<Arc<ResidentPageCache<ResidentNamingPage>>>,
}

impl std::fmt::Debug for ColdNamingStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ColdNamingStore")
            .field("branches", &self.state.branches.len())
            .finish_non_exhaustive()
    }
}

impl ColdNamingStore {
    /// Opens `state` over `source` without reading any page; reads are
    /// confined to the state's own page roster.
    pub fn open(
        state: Arc<CheckpointNamingState>,
        source: Arc<dyn RelationalDurableColdSource>,
    ) -> Self {
        let roster = state.page_members();
        Self {
            source: ConfinedMemberSource::new(
                ColdMemberSource::new(source, NAMING_PAGE_BYTES as u64),
                roster,
            ),
            group: PageEvictionGroup::new(),
            accountant: OnceLock::new(),
            cache: OnceLock::new(),
            state,
        }
    }

    /// Charges later faults to `accountant`; only effective before the first
    /// fault, after which pages already hold their charges.
    pub fn bind_working_set(&self, accountant: Arc<WorkingSetAccountant>) -> bool {
        self.cache.get().is_none() && self.accountant.set(accountant).is_ok()
    }

    pub fn state(&self) -> &Arc<CheckpointNamingState> {
        &self.state
    }

    pub fn observation(&self) -> PageCacheObservation {
        self.cache
            .get()
            .map(|cache| cache.observation())
            .unwrap_or_default()
    }

    fn cache(&self) -> &Arc<ResidentPageCache<ResidentNamingPage>> {
        self.cache.get_or_init(|| {
            let accountant = self.accountant.get().cloned().unwrap_or_else(|| {
                WorkingSetAccountant::new(ResidencyBudget {
                    head_working_set_bytes: u64::MAX,
                    historical_working_set_bytes: u64::MAX,
                    max_in_flight_fill_bytes: u64::MAX,
                    streaming_window_bytes: u32::MAX,
                })
            });
            self.group.cache(accountant, ResidencyPoolKind::Historical)
        })
    }

    /// The page `reference` names, faulted on a miss and checked against it.
    pub fn page(&self, reference: &NamingPageRef) -> Result<Arc<ResidentNamingPage>, Cause> {
        self.cache()
            .page_checked(&self.source, reference.member, |page| page.admit(reference))
    }

    /// The raw stored row bytes of `key` in `pages` (envelope-prefixed).
    pub(crate) fn raw_get(
        &self,
        pages: &[NamingPageRef],
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Cause> {
        let Some(slot) = page_for(pages, key) else {
            return Ok(None);
        };
        Ok(self.page(&pages[slot])?.value(key).map(<[u8]>::to_vec))
    }

    fn reassemble(
        &self,
        pages: &[NamingPageRef],
        key: &[u8],
        raw: &[u8],
    ) -> Result<Vec<u8>, Cause> {
        let Some((count, total)) = parse_value_envelope(raw)? else {
            return Ok(raw[1..].to_vec());
        };
        let mut out = Vec::new();
        out.try_reserve_exact(total)
            .map_err(|_| Cause::MemoryExhausted)?;
        for index in 0..count {
            let expected_len = if index + 1 == count {
                total - (index as usize) * MAX_INLINE_VALUE_BYTES
            } else {
                MAX_INLINE_VALUE_BYTES
            };
            let ckey = chunk_key(key, index);
            let chunk = self.raw_get(pages, &ckey)?.ok_or(Cause::Damaged)?;
            if chunk.len() != expected_len {
                return Err(Cause::Damaged);
            }
            out.extend_from_slice(&chunk);
        }
        Ok(out)
    }

    /// The value of row `key` in `pages`, faulting only the page (and any
    /// continuation chunk pages) holding it.
    ///
    /// Complexity: O(log pages + log rows) for inline values.
    pub fn get(&self, pages: &[NamingPageRef], key: &[u8]) -> Result<Option<Vec<u8>>, Cause> {
        let Some(raw) = self.raw_get(pages, key)? else {
            return Ok(None);
        };
        Ok(Some(self.reassemble(pages, key, &raw)?))
    }

    /// Visits every row of `pages` starting with `prefix`, ascending, one
    /// page resident at a time; `visit` returns `false` to stop.
    ///
    /// When a page contains any chunked row, its matching `(key, raw)` entries
    /// are staged and the logical page borrow is released before faulting any
    /// continuation chunk page, so chunked scans succeed under a 1-page budget.
    ///
    /// Complexity: O(log pages + rows visited), faulting only the pages the
    /// prefix range spans.
    pub fn scan(
        &self,
        pages: &[NamingPageRef],
        prefix: &[u8],
        mut visit: impl FnMut(&[u8], &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let start = pages.partition_point(|page| page.last.as_slice() < prefix);
        for reference in &pages[start..] {
            if !reference.first.starts_with(prefix) && reference.first.as_slice() > prefix {
                break;
            }
            let page = self.page(reference)?;
            let has_chunked = page
                .page
                .rows
                .iter()
                .any(|(k, r)| k.starts_with(prefix) && r.first() == Some(&VALUE_CHUNKED_TAG));
            if !has_chunked {
                for (key, raw) in &page.page.rows {
                    if key.as_slice() < prefix {
                        continue;
                    }
                    if !key.starts_with(prefix) {
                        return Ok(());
                    }
                    if parse_value_envelope(raw)?.is_some() {
                        return Err(Cause::Damaged);
                    }
                    if !visit(key, &raw[1..])? {
                        return Ok(());
                    }
                }
                continue;
            }
            let mut staged = Vec::new();
            let mut hit_end = false;
            for (key, raw) in &page.page.rows {
                if key.as_slice() < prefix {
                    continue;
                }
                if !key.starts_with(prefix) {
                    hit_end = true;
                    break;
                }
                staged.push((key.clone(), raw.clone()));
            }
            // Release the logical page borrow before faulting any chunk pages.
            drop(page);
            for (key, raw) in staged {
                let keep = match parse_value_envelope(&raw)? {
                    None => visit(&key, &raw[1..])?,
                    Some(_) => {
                        let full = self.reassemble(pages, &key, &raw)?;
                        visit(&key, &full)?
                    }
                };
                if !keep {
                    return Ok(());
                }
            }
            if hit_end {
                return Ok(());
            }
        }
        Ok(())
    }

    /// The exact bytes of page member `locator`, uncached, so a capture can
    /// copy a carried page the destination does not hold.
    pub fn read_member(&self, locator: RelationalDurableMemberLocator) -> Result<Vec<u8>, Cause> {
        let length = usize::try_from(locator.length).map_err(|_| Cause::MemoryExhausted)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| Cause::MemoryExhausted)?;
        bytes.resize(length, 0);
        let mut filled = 0;
        while filled < length {
            match self
                .source
                .read_range(locator, filled as u64, &mut bytes[filled..])?
            {
                0 => return Err(Cause::MissingMember),
                read => filled += read,
            }
        }
        Ok(bytes)
    }
}
