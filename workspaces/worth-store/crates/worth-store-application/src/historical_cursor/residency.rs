//! Isolated `Historical` vs `Head` residency working-set pool for retained-basis
//! historical cursor reads (Store Milestone 4, Phase M4.4).
//!
//! Concurrent cold fills reserve BOTH `max_in_flight_fill_bytes` and
//! `historical_working_set_bytes` atomically under the pool mutex BEFORE
//! releasing the mutex for I/O, holding an RAII [`InFlightFillReservation`]
//! guard until either committed to the cache or rolled back on error/drop.

use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use worth_relational::facade::config::ResidencyBudget;
use worth_runtime_world::facade::{HistoricalReadDenial, HistoricalResidencyPoolSnapshot};

use super::chunk_format::ChunkMemberId;

struct CachedHistoricalMember {
    bytes: Arc<Vec<u8>>,
    charged_bytes: u64,
}

#[derive(Default)]
struct HistoricalCacheState {
    entries: BTreeMap<ChunkMemberId, CachedHistoricalMember>,
    lru: VecDeque<ChunkMemberId>,
    cached_bytes: u64,
    pinned_borrow_bytes: u64,
    in_flight_fill_bytes: u64,
    evictions: u64,
}

impl HistoricalCacheState {
    fn total_historical_bytes(&self) -> u64 {
        self.cached_bytes
            .saturating_add(self.pinned_borrow_bytes)
            .saturating_add(self.in_flight_fill_bytes)
    }
}

/// Separate `Head` and `Historical` residency budgets for cursor reads.
#[derive(Clone)]
pub struct HistoricalWorkingSetPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    budget: ResidencyBudget,
    head_resident_bytes: AtomicU64,
    chunks_faulted: AtomicU64,
    continuation_pages_faulted: AtomicU64,
    manifest_pages_faulted: AtomicU64,
    historical: Mutex<HistoricalCacheState>,
}

/// Active borrow against the `Head` working set (never touched by historical scans).
pub struct HeadWorkingSetGuard {
    pool: Arc<PoolInner>,
    bytes: u64,
}

impl Drop for HeadWorkingSetGuard {
    fn drop(&mut self) {
        self.pool
            .head_resident_bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

/// Active borrow against the `Historical` working set (released immediately on `Drop`).
pub struct HistoricalPoolBorrow {
    pool: Arc<PoolInner>,
    bytes: u64,
}

impl Drop for HistoricalPoolBorrow {
    fn drop(&mut self) {
        let mut state = self
            .pool
            .historical
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pinned_borrow_bytes = state.pinned_borrow_bytes.saturating_sub(self.bytes);
    }
}

/// RAII reservation held across cold I/O to eliminate TOCTOU on concurrent fills.
struct InFlightFillReservation {
    pool: Arc<PoolInner>,
    bytes: u64,
    released: bool,
}

impl InFlightFillReservation {
    fn release_locked(&mut self, state: &mut HistoricalCacheState) {
        if !self.released {
            state.in_flight_fill_bytes = state.in_flight_fill_bytes.saturating_sub(self.bytes);
            self.released = true;
        }
    }
}

impl Drop for InFlightFillReservation {
    fn drop(&mut self) {
        if !self.released {
            let mut state = self
                .pool
                .historical
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.in_flight_fill_bytes = state.in_flight_fill_bytes.saturating_sub(self.bytes);
            self.released = true;
        }
    }
}

impl HistoricalWorkingSetPool {
    pub fn new(budget: ResidencyBudget) -> Self {
        Self {
            inner: Arc::new(PoolInner {
                budget,
                head_resident_bytes: AtomicU64::new(0),
                chunks_faulted: AtomicU64::new(0),
                continuation_pages_faulted: AtomicU64::new(0),
                manifest_pages_faulted: AtomicU64::new(0),
                historical: Mutex::new(HistoricalCacheState::default()),
            }),
        }
    }

    pub fn manifest_pages_faulted(&self) -> u64 {
        self.inner.manifest_pages_faulted.load(Ordering::Relaxed)
    }

    pub fn budget(&self) -> ResidencyBudget {
        self.inner.budget
    }

    /// Charges bytes against the `Head` working set. Historical cursor reads
    /// NEVER evict or reduce these resident bytes.
    pub fn pin_head_bytes(&self, bytes: u64) -> Option<HeadWorkingSetGuard> {
        let mut current = self.inner.head_resident_bytes.load(Ordering::Acquire);
        loop {
            let next = current.checked_add(bytes)?;
            if next > self.inner.budget.head_working_set_bytes {
                return None;
            }
            match self.inner.head_resident_bytes.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Some(HeadWorkingSetGuard {
                        pool: Arc::clone(&self.inner),
                        bytes,
                    });
                }
                Err(observed) => current = observed,
            }
        }
    }

    /// Explicitly borrows bytes from the `Historical` pool (e.g. an active
    /// in-flight cursor window). Evicts unborrowed historical cache entries first;
    /// returns `MemoryExhausted` if active borrows saturate the historical ceiling.
    pub fn borrow_historical_bytes(
        &self,
        bytes: u64,
    ) -> Result<HistoricalPoolBorrow, HistoricalReadDenial> {
        let allowance = self.inner.budget.historical_working_set_bytes;
        let mut state = self
            .inner
            .historical
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        evict_to_fit(&mut state, allowance, bytes)?;
        state.pinned_borrow_bytes = state.pinned_borrow_bytes.saturating_add(bytes);
        Ok(HistoricalPoolBorrow {
            pool: Arc::clone(&self.inner),
            bytes,
        })
    }

    /// Faults or serves a historical chunk/continuation member strictly within
    /// BOTH `ResidencyBudget::max_in_flight_fill_bytes` and
    /// `ResidencyBudget::historical_working_set_bytes` with zero TOCTOU race.
    pub fn load_historical_member<F>(
        &self,
        id: ChunkMemberId,
        is_continuation: bool,
        fetch: F,
    ) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>
    where
        F: FnOnce() -> Result<Vec<u8>, HistoricalReadDenial>,
    {
        self.load_historical_entry(id, Some(is_continuation), fetch)
    }

    /// Faults or serves one per-basis `BasisManifest` member on demand strictly
    /// within the isolated `Historical` residency budget (without counting as a
    /// data chunk fault).
    pub(super) fn load_historical_manifest<F>(
        &self,
        id: ChunkMemberId,
        fetch: F,
    ) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>
    where
        F: FnOnce() -> Result<Vec<u8>, HistoricalReadDenial>,
    {
        self.load_historical_entry(id, None, fetch)
    }

    fn load_historical_entry<F>(
        &self,
        id: ChunkMemberId,
        chunk_fault_kind: Option<bool>,
        fetch: F,
    ) -> Result<Arc<Vec<u8>>, HistoricalReadDenial>
    where
        F: FnOnce() -> Result<Vec<u8>, HistoricalReadDenial>,
    {
        let allowance = self.inner.budget.historical_working_set_bytes;
        let max_in_flight = self.inner.budget.max_in_flight_fill_bytes;
        let needed = id.length.max(1);

        let mut reservation = {
            let mut state = self
                .inner
                .historical
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(entry) = state.entries.get(&id) {
                let bytes = Arc::clone(&entry.bytes);
                touch_lru(&mut state.lru, id);
                return Ok(bytes);
            }
            if state.in_flight_fill_bytes.saturating_add(needed) > max_in_flight {
                return Err(HistoricalReadDenial::MemoryExhausted {
                    requested_bytes: needed,
                    allowance_bytes: max_in_flight,
                });
            }
            evict_to_fit(&mut state, allowance, needed)?;
            state.in_flight_fill_bytes = state.in_flight_fill_bytes.saturating_add(needed);
            InFlightFillReservation {
                pool: Arc::clone(&self.inner),
                bytes: needed,
                released: false,
            }
        };

        // Run I/O outside the lock while `reservation` atomically holds its
        // in-flight & working-set allowance; automatically rolls back on error/panic.
        let raw = fetch()?;
        match chunk_fault_kind {
            Some(true) => {
                self.inner
                    .continuation_pages_faulted
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(false) => {
                self.inner.chunks_faulted.fetch_add(1, Ordering::Relaxed);
            }
            None => {
                self.inner
                    .manifest_pages_faulted
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
        let charged = (raw.len() as u64).max(1);
        let shared = Arc::new(raw);

        let mut state = self
            .inner
            .historical
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        reservation.release_locked(&mut state);
        if let Some(existing) = state.entries.get(&id) {
            return Ok(Arc::clone(&existing.bytes));
        }
        evict_to_fit(&mut state, allowance, charged)?;
        state.entries.insert(
            id,
            CachedHistoricalMember {
                bytes: Arc::clone(&shared),
                charged_bytes: charged,
            },
        );
        state.lru.push_back(id);
        state.cached_bytes = state.cached_bytes.saturating_add(charged);
        Ok(shared)
    }

    /// Current residency snapshot separating `Head` and `Historical` pools.
    pub fn snapshot(&self) -> HistoricalResidencyPoolSnapshot {
        let state = self
            .inner
            .historical
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut borrowed_cache = 0u64;
        for entry in state.entries.values() {
            if Arc::strong_count(&entry.bytes) > 1 {
                borrowed_cache = borrowed_cache.saturating_add(entry.charged_bytes);
            }
        }
        HistoricalResidencyPoolSnapshot {
            head_resident_bytes: self.inner.head_resident_bytes.load(Ordering::Acquire),
            head_allowance_bytes: self.inner.budget.head_working_set_bytes,
            historical_resident_bytes: state.cached_bytes.saturating_add(state.pinned_borrow_bytes),
            historical_borrowed_bytes: borrowed_cache.saturating_add(state.pinned_borrow_bytes),
            historical_allowance_bytes: self.inner.budget.historical_working_set_bytes,
            in_flight_fill_bytes: state.in_flight_fill_bytes,
            historical_evictions: state.evictions,
            chunks_faulted: self.inner.chunks_faulted.load(Ordering::Relaxed),
            continuation_pages_faulted: self
                .inner
                .continuation_pages_faulted
                .load(Ordering::Relaxed),
        }
    }
}

fn touch_lru(lru: &mut VecDeque<ChunkMemberId>, id: ChunkMemberId) {
    if let Some(pos) = lru.iter().position(|existing| *existing == id) {
        lru.remove(pos);
    }
    lru.push_back(id);
}

fn evict_to_fit(
    state: &mut HistoricalCacheState,
    allowance: u64,
    needed: u64,
) -> Result<(), HistoricalReadDenial> {
    if needed > allowance {
        return Err(HistoricalReadDenial::MemoryExhausted {
            requested_bytes: needed,
            allowance_bytes: allowance,
        });
    }
    if state.total_historical_bytes().saturating_add(needed) <= allowance {
        return Ok(());
    }
    let mut scans = state.lru.len();
    while state.total_historical_bytes().saturating_add(needed) > allowance && scans > 0 {
        scans -= 1;
        let Some(victim_id) = state.lru.pop_front() else {
            break;
        };
        let evictable = state
            .entries
            .get(&victim_id)
            .is_some_and(|entry| Arc::strong_count(&entry.bytes) == 1);
        if evictable {
            if let Some(removed) = state.entries.remove(&victim_id) {
                state.cached_bytes = state.cached_bytes.saturating_sub(removed.charged_bytes);
                state.evictions = state.evictions.saturating_add(1);
            }
        } else {
            state.lru.push_back(victim_id);
        }
    }
    if state.total_historical_bytes().saturating_add(needed) <= allowance {
        Ok(())
    } else {
        Err(HistoricalReadDenial::MemoryExhausted {
            requested_bytes: needed,
            allowance_bytes: allowance,
        })
    }
}
