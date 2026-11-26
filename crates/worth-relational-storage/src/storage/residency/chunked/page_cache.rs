//! Bounded on-demand cache of decoded content-addressed pages.
//!
//! Every resident page holds a [`WorkingSetCharge`] in its cache slot. A page
//! is faulted only after its encoded bytes win an in-flight fill reservation
//! and its decoded bytes win a pool reservation. Under pressure the cache
//! evicts its own least-recently used pages that no reader still borrows
//! (a borrowed page's bytes are still live, so dropping its charge would
//! under-account), then refuses with `MemoryExhausted`.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::member_io::{load_member, ChunkMemberSource};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
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

pub struct ResidentPageCache<T> {
    accountant: Arc<WorkingSetAccountant>,
    pool: ResidencyPoolKind,
    slots: Mutex<PageSlots<T>>,
}

struct PageSlots<T> {
    pages: HashMap<RelationalDurableMemberLocator, CachedPage<T>>,
    clock: u64,
    faults: u64,
    evictions: u64,
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

impl<T: ResidentPage> ResidentPageCache<T> {
    pub fn new(accountant: Arc<WorkingSetAccountant>, pool: ResidencyPoolKind) -> Self {
        Self {
            accountant,
            pool,
            slots: Mutex::new(PageSlots {
                pages: HashMap::new(),
                clock: 0,
                faults: 0,
                evictions: 0,
            }),
        }
    }

    /// The resident page `locator` names, faulting it from `source` on a miss.
    pub fn page(
        &self,
        source: &dyn ChunkMemberSource,
        locator: RelationalDurableMemberLocator,
    ) -> Result<Arc<T>, Cause> {
        if let Some(page) = self.lock().touch(locator) {
            return Ok(page);
        }
        let loaded = load_member(&self.accountant, source, locator)?;
        let page = Arc::new(T::decode(&loaded.bytes, locator)?);
        drop(loaded);
        let charge = self.reserve_resident(page.resident_bytes())?;
        let mut slots = self.lock();
        if let Some(raced) = slots.touch(locator) {
            return Ok(raced);
        }
        slots.faults += 1;
        slots.clock += 1;
        let last_use = slots.clock;
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

    /// Reserves pool bytes, evicting this cache's unborrowed pages oldest
    /// first while the pool refuses.
    fn reserve_resident(&self, bytes: u64) -> Result<WorkingSetCharge, Cause> {
        loop {
            match self.accountant.reserve(self.pool, bytes, std::ptr::null()) {
                Ok(charge) => return Ok(charge),
                Err(_) if self.lock().evict_one() => continue,
                Err(_) => return Err(Cause::MemoryExhausted),
            }
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, PageSlots<T>> {
        self.slots.lock().unwrap_or_else(|error| error.into_inner())
    }
}

impl<T> PageSlots<T> {
    fn touch(&mut self, locator: RelationalDurableMemberLocator) -> Option<Arc<T>> {
        self.clock += 1;
        let clock = self.clock;
        let cached = self.pages.get_mut(&locator)?;
        cached.last_use = clock;
        Some(Arc::clone(&cached.page))
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
