//! Bounded on-demand cache of decoded content-addressed pages.
//!
//! Every resident page holds a [`WorkingSetCharge`] in its cache slot. A page
//! is faulted only after its encoded bytes win an in-flight fill reservation
//! and its decoded bytes win a pool reservation. Under pressure the cache
//! evicts its own least-recently used pages that no reader still borrows
//! (a borrowed page's bytes are still live, so dropping its charge would
//! under-account), then refuses with `MemoryExhausted`.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use super::member_io::{load_member, ChunkMemberSource};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::evictor::ExternalPoolEvictor;
use crate::storage::residency::working_set::{
    ResidencyPoolKind, WorkingSetAccountant, WorkingSetCharge,
};

/// A decoded page that knows its resident footprint.
pub trait ResidentPage: Sized {
    /// Decodes and authenticates the member `locator` names.
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause>;
    /// Heap and inline bytes this decoded page keeps alive.
    fn resident_bytes(&self) -> u64;
}

trait EvictableCache: Send + Sync {
    fn pool(&self) -> ResidencyPoolKind;
    fn oldest_evictable_use(&self) -> Option<u64>;
    fn evict_one_unborrowed(&self) -> bool;
}

/// Shared LRU clock and peer registry for caches that share one working-set
/// pool, so unborrowed pages in one cache never starve sibling caches.
///
/// Peers are held weakly; registering a cache never extends its lifetime.
pub struct PageEvictionGroup {
    clock: AtomicU64,
    peers: Mutex<Vec<Weak<dyn EvictableCache>>>,
}

impl std::fmt::Debug for PageEvictionGroup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PageEvictionGroup")
            .finish_non_exhaustive()
    }
}

impl ExternalPoolEvictor for PageEvictionGroup {
    fn evict_one_for_pool(&self, pool: ResidencyPoolKind) -> bool {
        self.evict_oldest_in_pool(Some(pool))
    }
}

impl PageEvictionGroup {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clock: AtomicU64::new(0),
            peers: Mutex::new(Vec::new()),
        })
    }

    pub fn cache<T: ResidentPage + Send + Sync + 'static>(
        self: &Arc<Self>,
        accountant: Arc<WorkingSetAccountant>,
        pool: ResidencyPoolKind,
    ) -> Arc<ResidentPageCache<T>> {
        accountant.enroll_external_evictor(Arc::downgrade(
            &(Arc::clone(self) as Arc<dyn ExternalPoolEvictor>),
        ));
        let cache = Arc::new(ResidentPageCache {
            accountant,
            pool,
            group: Some(Arc::clone(self)),
            slots: Mutex::new(PageSlots::default()),
        });
        let weak: Weak<dyn EvictableCache> =
            Arc::downgrade(&(Arc::clone(&cache) as Arc<dyn EvictableCache>));
        self.peers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(weak);
        cache
    }

    fn next_tick(&self) -> u64 {
        self.clock.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Evicts the globally least-recently-used unborrowed page across all
    /// live peer caches without holding `self.peers` across any cache lock.
    fn evict_oldest(&self, pool: ResidencyPoolKind) -> bool {
        self.evict_oldest_in_pool(Some(pool))
    }

    fn evict_oldest_in_pool(&self, pool: Option<ResidencyPoolKind>) -> bool {
        let live: Vec<Arc<dyn EvictableCache>> = {
            let mut peers = self.peers.lock().unwrap_or_else(|error| error.into_inner());
            let mut upgraded = Vec::with_capacity(peers.len());
            peers.retain(|weak| match weak.upgrade() {
                Some(cache) => {
                    if pool.is_none_or(|target| cache.pool() == target) {
                        upgraded.push(cache);
                    }
                    true
                }
                None => false,
            });
            upgraded
        };
        let oldest = live
            .iter()
            .enumerate()
            .filter_map(|(index, peer)| peer.oldest_evictable_use().map(|tick| (tick, index)))
            .min_by_key(|(tick, _)| *tick)
            .map(|(_, index)| index);
        if let Some(index) = oldest
            && live[index].evict_one_unborrowed()
        {
            return true;
        }
        // Fallback if a concurrent thread took or borrowed the oldest victim.
        live.iter().any(|peer| peer.evict_one_unborrowed())
    }
}

pub struct ResidentPageCache<T> {
    accountant: Arc<WorkingSetAccountant>,
    pool: ResidencyPoolKind,
    group: Option<Arc<PageEvictionGroup>>,
    slots: Mutex<PageSlots<T>>,
}

struct PageSlots<T> {
    pages: HashMap<RelationalDurableMemberLocator, CachedPage<T>>,
    clock: u64,
    faults: u64,
    evictions: u64,
}

impl<T> Default for PageSlots<T> {
    fn default() -> Self {
        Self {
            pages: HashMap::new(),
            clock: 0,
            faults: 0,
            evictions: 0,
        }
    }
}

struct CachedPage<T> {
    last_use: u64,
    page: Arc<T>,
    _charge: WorkingSetCharge,
}

/// Fault and eviction counts of one cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PageCacheObservation {
    pub resident_pages: usize,
    pub faults: u64,
    pub evictions: u64,
}

impl<T: ResidentPage + Send + Sync + 'static> EvictableCache for ResidentPageCache<T> {
    fn pool(&self) -> ResidencyPoolKind {
        self.pool
    }

    fn oldest_evictable_use(&self) -> Option<u64> {
        self.lock().oldest_evictable_use()
    }

    fn evict_one_unborrowed(&self) -> bool {
        self.lock().evict_one()
    }
}

impl<T: ResidentPage> ResidentPageCache<T> {
    pub fn new(accountant: Arc<WorkingSetAccountant>, pool: ResidencyPoolKind) -> Self {
        Self {
            accountant,
            pool,
            group: None,
            slots: Mutex::new(PageSlots::default()),
        }
    }

    /// The resident page `locator` names, faulting it from `source` on a miss.
    pub fn page(
        &self,
        source: &dyn ChunkMemberSource,
        locator: RelationalDurableMemberLocator,
    ) -> Result<Arc<T>, Cause> {
        let tick = self.next_tick();
        if let Some(page) = self.lock().touch(locator, tick) {
            return Ok(page);
        }
        let loaded = load_member(&self.accountant, source, locator)?;
        let page = Arc::new(T::decode(&loaded.bytes, locator)?);
        drop(loaded);
        let charge = self.reserve_resident(page.resident_bytes())?;
        let tick = self.next_tick();
        let mut slots = self.lock();
        if let Some(raced) = slots.touch(locator, tick) {
            return Ok(raced);
        }
        slots.faults += 1;
        let last_use = tick.unwrap_or_else(|| {
            slots.clock += 1;
            slots.clock
        });
        slots.pages.insert(
            locator,
            CachedPage {
                last_use,
                page: Arc::clone(&page),
                _charge: charge,
            },
        );
        Ok(page)
    }

    pub fn observation(&self) -> PageCacheObservation {
        let slots = self.lock();
        PageCacheObservation {
            resident_pages: slots.pages.len(),
            faults: slots.faults,
            evictions: slots.evictions,
        }
    }

    fn next_tick(&self) -> Option<u64> {
        self.group.as_ref().map(|group| group.next_tick())
    }

    fn pool_ceiling(&self) -> u64 {
        let budget = self.accountant.budget();
        match self.pool {
            ResidencyPoolKind::Head => budget.head_working_set_bytes,
            ResidencyPoolKind::Historical => budget.historical_working_set_bytes,
        }
    }

    /// Reserves pool bytes, evicting unborrowed pages oldest first across this
    /// cache's eviction group while the pool refuses.
    fn reserve_resident(&self, bytes: u64) -> Result<WorkingSetCharge, Cause> {
        if bytes > self.pool_ceiling() {
            return Err(Cause::MemoryExhausted);
        }
        loop {
            match self.accountant.reserve(self.pool, bytes, std::ptr::null()) {
                Ok(charge) => return Ok(charge),
                Err(_) if self.evict_one() => continue,
                Err(_) => return Err(Cause::MemoryExhausted),
            }
        }
    }

    fn evict_one(&self) -> bool {
        match &self.group {
            Some(group) => group.evict_oldest(self.pool),
            None => self.lock().evict_one(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PageSlots<T>> {
        self.slots.lock().unwrap_or_else(|error| error.into_inner())
    }
}

impl<T> PageSlots<T> {
    fn touch(
        &mut self,
        locator: RelationalDurableMemberLocator,
        tick: Option<u64>,
    ) -> Option<Arc<T>> {
        let clock = tick.unwrap_or_else(|| {
            self.clock += 1;
            self.clock
        });
        let cached = self.pages.get_mut(&locator)?;
        cached.last_use = clock;
        Some(Arc::clone(&cached.page))
    }

    fn oldest_evictable_use(&self) -> Option<u64> {
        self.pages
            .values()
            .filter(|cached| Arc::strong_count(&cached.page) == 1)
            .map(|cached| cached.last_use)
            .min()
    }

    /// Evicts the least recently used page no reader borrows.
    fn evict_one(&mut self) -> bool {
        let Some(victim) = self
            .pages
            .iter()
            .filter(|(_, cached)| Arc::strong_count(&cached.page) == 1)
            .min_by_key(|(_, cached)| cached.last_use)
            .map(|(locator, _)| *locator)
        else {
            return false;
        };
        // The charge drops with the slot, after its last strong reference.
        drop(self.pages.remove(&victim));
        self.evictions += 1;
        true
    }
}
