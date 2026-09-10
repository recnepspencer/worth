use std::sync::{Arc, Mutex, PoisonError};

use crate::runtime::{
    WorthQueryConditionalEvaluationCacheBudget, WorthQueryConditionalEvaluationCacheBudgetDenial,
};

use super::retention::{
    WorthQueryConditionalEvaluationRetentionLedger,
    WorthQueryConditionalEvaluationRetentionReservation,
};

pub(super) trait WorthQueryIdleCacheEntry {
    fn is_idle(entry: &Arc<Self>) -> bool;
}

pub(super) struct WorthQueryBoundedIdleCache<E> {
    maximum_entries: usize,
    entry_retained_bytes: u64,
    retention: Arc<WorthQueryConditionalEvaluationRetentionLedger>,
    state: Mutex<WorthQueryBoundedIdleCacheState<E>>,
}

struct WorthQueryBoundedIdleCacheState<E> {
    entries: Vec<WorthQueryCachedEntry<E>>,
    _fixed_retention: WorthQueryConditionalEvaluationRetentionReservation,
    next_use: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
}

struct WorthQueryCachedEntry<E> {
    value: Arc<E>,
    _retention: WorthQueryConditionalEvaluationRetentionReservation,
    last_use: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorthQueryIdleCacheCapacity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runtime) struct WorthQueryConditionalEvaluationCacheObservation {
    pub(in crate::runtime) retained_entries: usize,
    pub(in crate::runtime) retained_bytes: u64,
    pub(in crate::runtime) hits: u64,
    pub(in crate::runtime) misses: u64,
    pub(in crate::runtime) evictions: u64,
}

impl<E: WorthQueryIdleCacheEntry> WorthQueryBoundedIdleCache<E> {
    pub(super) fn new(
        budget: WorthQueryConditionalEvaluationCacheBudget,
        entry_retained_bytes: u64,
    ) -> Result<Self, WorthQueryConditionalEvaluationCacheBudgetDenial> {
        let maximum_entries = budget.maximum_retained_entries();
        let ledger_bytes = super::budget::retention_ledger_allocation_charge().map_err(|_| {
            WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow
        })?;
        let requested_backing_bytes =
            super::budget::vector_backing_charge::<WorthQueryCachedEntry<E>>(maximum_entries)
                .map_err(|_| {
                    WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow
                })?;
        budget.require_bytes(total_retained_bytes(
            maximum_entries,
            entry_retained_bytes,
            requested_backing_bytes,
            ledger_bytes,
        )?)?;

        let mut entries = Vec::new();
        entries.try_reserve_exact(maximum_entries).map_err(|_| {
            WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedAllocationUnavailable {
                requested_entries: maximum_entries,
            }
        })?;
        let actual_backing_bytes =
            super::budget::vector_backing_charge::<WorthQueryCachedEntry<E>>(entries.capacity())
                .map_err(|_| {
                    WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow
                })?;
        budget.require_bytes(total_retained_bytes(
            maximum_entries,
            entry_retained_bytes,
            actual_backing_bytes,
            ledger_bytes,
        )?)?;

        let retention = WorthQueryConditionalEvaluationRetentionLedger::new(budget);
        let fixed_retention = retention
            .reserve(
                0,
                actual_backing_bytes
                    .checked_add(ledger_bytes)
                    .expect("validated fixed Query cache charges fit in u64"),
            )
            .expect("validated Query cache budget covers its fixed retention");
        Ok(Self {
            maximum_entries,
            entry_retained_bytes,
            retention,
            state: Mutex::new(WorthQueryBoundedIdleCacheState {
                entries,
                _fixed_retention: fixed_retention,
                next_use: 0,
                hits: 0,
                misses: 0,
                evictions: 0,
            }),
        })
    }

    pub(super) fn find_or_insert(
        &self,
        matches: impl Fn(&E) -> bool,
        create: impl FnOnce() -> E,
    ) -> Result<Arc<E>, WorthQueryIdleCacheCapacity> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let use_ordinal = state.take_use_ordinal();
        if let Some(index) = state.entries.iter().position(|entry| matches(&entry.value)) {
            state.hits = state.hits.saturating_add(1);
            state.entries[index].last_use = use_ordinal;
            return Ok(Arc::clone(&state.entries[index].value));
        }
        state.misses = state.misses.saturating_add(1);
        if state.entries.len() == self.maximum_entries && !state.evict_oldest_idle(None) {
            return Err(WorthQueryIdleCacheCapacity);
        }
        let retention = self
            .retention
            .reserve(1, self.entry_retained_bytes)
            .expect("validated fixed cache capacity covers each retained entry");
        let value = Arc::new(create());
        state.entries.push(WorthQueryCachedEntry {
            value: Arc::clone(&value),
            _retention: retention,
            last_use: use_ordinal,
        });
        Ok(value)
    }

    pub(super) fn evict_oldest_idle_except(&self, retained: &Arc<E>) -> bool {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .evict_oldest_idle(Some(retained))
    }

    pub(super) fn observe(&self) -> WorthQueryConditionalEvaluationCacheObservation {
        let retention = self.retention.observe();
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        WorthQueryConditionalEvaluationCacheObservation {
            retained_entries: retention.entries,
            retained_bytes: retention.bytes,
            hits: state.hits,
            misses: state.misses,
            evictions: state.evictions,
        }
    }
}

fn total_retained_bytes(
    maximum_entries: usize,
    entry_retained_bytes: u64,
    backing_bytes: u64,
    ledger_bytes: u64,
) -> Result<u64, WorthQueryConditionalEvaluationCacheBudgetDenial> {
    let entries = u64::try_from(maximum_entries)
        .ok()
        .and_then(|count| entry_retained_bytes.checked_mul(count))
        .ok_or(WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow)?;
    entries
        .checked_add(backing_bytes)
        .and_then(|total| total.checked_add(ledger_bytes))
        .ok_or(WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow)
}

impl<E: WorthQueryIdleCacheEntry> WorthQueryBoundedIdleCacheState<E> {
    fn take_use_ordinal(&mut self) -> u64 {
        let current = self.next_use;
        self.next_use = self.next_use.wrapping_add(1);
        current
    }

    fn evict_oldest_idle(&mut self, retained: Option<&Arc<E>>) -> bool {
        let candidate = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                retained.is_none_or(|retained| !Arc::ptr_eq(&entry.value, retained))
                    && E::is_idle(&entry.value)
            })
            .min_by_key(|(_, entry)| entry.last_use)
            .map(|(index, _)| index);
        candidate.is_some_and(|index| {
            self.entries.remove(index);
            self.evictions = self.evictions.saturating_add(1);
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Entry(usize);

    impl WorthQueryIdleCacheEntry for Entry {
        fn is_idle(entry: &Arc<Self>) -> bool {
            Arc::strong_count(entry) == 1
        }
    }

    fn cache(maximum_entries: usize) -> WorthQueryBoundedIdleCache<Entry> {
        WorthQueryBoundedIdleCache::new(
            WorthQueryConditionalEvaluationCacheBudget::bounded(maximum_entries, 16_384).unwrap(),
            64,
        )
        .unwrap()
    }

    #[test]
    fn active_entries_are_protected_and_idle_entries_release_their_reservation() {
        let cache = cache(1);
        let active = cache
            .find_or_insert(|entry| entry.0 == 1, || Entry(1))
            .unwrap();
        let before = cache.observe();
        assert_eq!(before.retained_entries, 1);
        assert!(cache
            .find_or_insert(|entry| entry.0 == 2, || Entry(2))
            .is_err());
        drop(active);
        drop(
            cache
                .find_or_insert(|entry| entry.0 == 2, || Entry(2))
                .unwrap(),
        );
        let after = cache.observe();
        assert_eq!(after.retained_entries, 1);
        assert_eq!(after.evictions, 1);
        assert_eq!(after.retained_bytes, before.retained_bytes);
    }

    #[test]
    fn warm_reuse_and_turnover_plateau_at_installed_capacity() {
        let cache = cache(2);
        drop(
            cache
                .find_or_insert(|entry| entry.0 == 0, || Entry(0))
                .unwrap(),
        );
        drop(
            cache
                .find_or_insert(|entry| entry.0 == 0, || Entry(0))
                .unwrap(),
        );
        assert_eq!(cache.observe().hits, 1);
        for key in 1..64 {
            drop(
                cache
                    .find_or_insert(|entry| entry.0 == key, || Entry(key))
                    .unwrap(),
            );
            assert!(cache.observe().retained_entries <= 2);
        }
        let final_state = cache.observe();
        assert_eq!(final_state.retained_entries, 2);
        assert_eq!(final_state.misses, 64);
        assert_eq!(final_state.evictions, 62);
    }

    #[test]
    fn configured_bytes_must_cover_backing_and_every_retained_entry() {
        const MAXIMUM_ENTRIES: usize = 2;
        const ENTRY_BYTES: u64 = 64;
        let exact_bytes = independent_required_bytes(MAXIMUM_ENTRIES, ENTRY_BYTES);
        let cache = WorthQueryBoundedIdleCache::<Entry>::new(
            WorthQueryConditionalEvaluationCacheBudget::bounded(MAXIMUM_ENTRIES, exact_bytes)
                .unwrap(),
            ENTRY_BYTES,
        )
        .expect("the exact independently calculated budget must be accepted");
        assert_eq!(
            cache.observe().retained_bytes,
            exact_bytes - ENTRY_BYTES * MAXIMUM_ENTRIES as u64
        );

        let denial = WorthQueryBoundedIdleCache::<Entry>::new(
            WorthQueryConditionalEvaluationCacheBudget::bounded(MAXIMUM_ENTRIES, exact_bytes - 1)
                .unwrap(),
            ENTRY_BYTES,
        )
        .err()
        .expect("one byte below the exact requirement must be denied");
        assert!(matches!(
            denial,
            WorthQueryConditionalEvaluationCacheBudgetDenial::InsufficientRetainedBytes {
                required,
                available,
            } if required == exact_bytes && available == exact_bytes - 1
        ));

        let overflow = WorthQueryBoundedIdleCache::<Entry>::new(
            WorthQueryConditionalEvaluationCacheBudget::bounded(usize::MAX, 1).unwrap(),
            ENTRY_BYTES,
        )
        .err()
        .expect("an unrepresentable capacity must be denied before allocation");
        assert_eq!(
            overflow,
            WorthQueryConditionalEvaluationCacheBudgetDenial::RetainedByteChargeOverflow
        );
    }

    fn independent_required_bytes(maximum_entries: usize, entry_bytes: u64) -> u64 {
        let vector_backing =
            std::mem::size_of::<WorthQueryCachedEntry<Entry>>() as u64 * maximum_entries as u64;
        let word = std::mem::size_of::<usize>() as u64;
        let ledger_payload =
            std::mem::size_of::<WorthQueryConditionalEvaluationRetentionLedger>() as u64;
        let ledger_alignment =
            std::mem::align_of::<WorthQueryConditionalEvaluationRetentionLedger>()
                .max(std::mem::align_of::<usize>()) as u64;
        let ledger_allocation = 2 * word + ledger_payload + 2 * ledger_alignment;
        vector_backing + ledger_allocation + entry_bytes * maximum_entries as u64
    }

    #[test]
    fn dropping_the_cache_releases_backing_and_entry_reservations() {
        let cache = cache(2);
        let ledger = Arc::clone(&cache.retention);
        drop(
            cache
                .find_or_insert(|entry| entry.0 == 1, || Entry(1))
                .unwrap(),
        );
        assert_eq!(ledger.observe().entries, 1);
        assert!(ledger.observe().bytes > 0);
        drop(cache);
        assert_eq!(ledger.observe().entries, 0);
        assert_eq!(ledger.observe().bytes, 0);
    }
}
