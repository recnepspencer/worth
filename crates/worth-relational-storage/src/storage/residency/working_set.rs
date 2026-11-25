//! Bounded working-set custody for cold-backed region cells (Store M4 D1).
//!
//! Every resident region a bound cell publishes carries a [`WorkingSetCharge`]
//! inside its own allocation, so its pool credit returns only when the last
//! strong `Arc<ResidentRegion>` drops. A lease count of zero only nominates a
//! victim; it never proves that bytes were freed.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub mod evictor;

const EXPANSION_UNIT: u64 = 1024;
/// Bounded re-sweeps of one reservation under concurrent eviction.
const EVICTION_ROUNDS: usize = 16;

/// Declarative residency ceilings. Head and historical working sets are
/// separate pools, so historical reads can never evict or starve head cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidencyBudget {
    pub head_working_set_bytes: u64,
    pub historical_working_set_bytes: u64,
    /// Encoded member bytes concurrently being read from cold storage.
    pub max_in_flight_fill_bytes: u64,
    /// Upper bound on one rolling-window step, in bytes of declared demand.
    pub streaming_window_bytes: u32,
}

/// The pool a bound cell draws from, fixed when the cell is bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidencyPoolKind {
    Head,
    Historical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidencyLevelKind {
    Head,
    Full,
}

/// Exact immutable basis a resident region must prove before it is served.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasisIdentity {
    pub content_digest: [u8; 32],
    pub schema_carrier_digest: [u8; 32],
    pub level: ResidencyLevelKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResidencyPressureSnapshot {
    pub head_resident_bytes: u64,
    pub head_borrowed_bytes: u64,
    pub historical_resident_bytes: u64,
    pub historical_borrowed_bytes: u64,
    pub raw_load_cache_bytes: u64,
    pub in_flight_fill_bytes: u64,
    pub evictions_completed: u64,
    pub busy_victim_skips: u64,
}

#[derive(Debug)]
struct PoolLedger {
    ceiling: u64,
    charged: AtomicU64,
}

impl PoolLedger {
    fn new(ceiling: u64) -> Self {
        Self {
            ceiling,
            charged: AtomicU64::new(0),
        }
    }

    fn try_add(&self, bytes: u64) -> bool {
        let mut current = self.charged.load(Ordering::Acquire);
        loop {
            let Some(next) = current
                .checked_add(bytes)
                .filter(|next| *next <= self.ceiling)
            else {
                return false;
            };
            match self.charged.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(observed) => current = observed,
            }
        }
    }

    fn release(&self, bytes: u64) {
        let previous = self.charged.fetch_sub(bytes, Ordering::AcqRel);
        debug_assert!(previous >= bytes, "a charge releases only bytes it added");
    }

    fn available(&self) -> u64 {
        self.ceiling
            .saturating_sub(self.charged.load(Ordering::Acquire))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LedgerKind {
    Pool(ResidencyPoolKind),
    InFlight,
}

/// One home's working-set authority. Cells hold it strongly; it holds cells
/// only weakly, so binding never extends a cell's lifetime.
#[derive(Debug)]
pub struct WorkingSetAccountant {
    budget: ResidencyBudget,
    head: PoolLedger,
    historical: PoolLedger,
    in_flight: PoolLedger,
    ring: Mutex<evictor::ClockRing>,
    evictions: AtomicU64,
    busy_skips: AtomicU64,
    /// Largest observed decoded/encoded ratio per level, in 1/1024 units.
    /// Pre-lock estimates scale encoded member lengths by it; loads then
    /// charge exactly what they measure.
    expansion: [AtomicU64; 2],
}

/// Bytes held against one ledger until dropped. Never cloned: one charge has
/// exactly one owner, normally the resident region allocation it funds.
#[derive(Debug)]
pub struct WorkingSetCharge {
    accountant: Arc<WorkingSetAccountant>,
    ledger: LedgerKind,
    bytes: u64,
}

/// Refusal of genuine saturation: no evictable victim could free the bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingSetExhausted {
    pub requested: u64,
    pub available: u64,
}

impl WorkingSetAccountant {
    pub fn new(budget: ResidencyBudget) -> Arc<Self> {
        Arc::new(Self {
            budget,
            head: PoolLedger::new(budget.head_working_set_bytes),
            historical: PoolLedger::new(budget.historical_working_set_bytes),
            in_flight: PoolLedger::new(budget.max_in_flight_fill_bytes),
            ring: Mutex::new(evictor::ClockRing::default()),
            evictions: AtomicU64::new(0),
            busy_skips: AtomicU64::new(0),
            expansion: [
                AtomicU64::new(EXPANSION_UNIT),
                AtomicU64::new(EXPANSION_UNIT),
            ],
        })
    }

    pub fn budget(&self) -> ResidencyBudget {
        self.budget
    }

    fn ledger(&self, kind: LedgerKind) -> &PoolLedger {
        match kind {
            LedgerKind::Pool(ResidencyPoolKind::Head) => &self.head,
            LedgerKind::Pool(ResidencyPoolKind::Historical) => &self.historical,
            LedgerKind::InFlight => &self.in_flight,
        }
    }

    /// Estimated resident bytes for `encoded` member bytes, from the largest
    /// decode expansion this home has measured so far at `level`.
    pub fn estimate(&self, level: ResidencyLevelKind, encoded: u64) -> u64 {
        let own = self.expansion[level as usize].load(Ordering::Acquire);
        // Until a Full image is measured, history decodes like the head.
        let ratio = if own == EXPANSION_UNIT {
            self.expansion[ResidencyLevelKind::Head as usize].load(Ordering::Acquire)
        } else {
            own
        };
        u64::try_from(u128::from(encoded) * u128::from(ratio) / u128::from(EXPANSION_UNIT))
            .unwrap_or(u64::MAX)
    }

    pub(super) fn observe_expansion(&self, level: ResidencyLevelKind, encoded: u64, decoded: u64) {
        if encoded == 0 {
            return;
        }
        let ratio = u128::from(decoded) * u128::from(EXPANSION_UNIT) / u128::from(encoded) + 1;
        self.expansion[level as usize]
            .fetch_max(u64::try_from(ratio).unwrap_or(u64::MAX), Ordering::AcqRel);
    }

    /// Charged bytes of live resident allocations in `pool`.
    pub fn charged_bytes(&self, pool: ResidencyPoolKind) -> u64 {
        self.ledger(LedgerKind::Pool(pool))
            .charged
            .load(Ordering::Acquire)
    }

    /// Reserve working-set bytes. Callers must not hold any cell `state`
    /// mutex except their own demand target, which is excluded from victims;
    /// victims are only ever taken with `try_lock`.
    pub fn reserve(
        self: &Arc<Self>,
        pool: ResidencyPoolKind,
        bytes: u64,
        requester: *const super::RelationalRegionResidency,
    ) -> Result<WorkingSetCharge, WorkingSetExhausted> {
        let ledger = self.ledger(LedgerKind::Pool(pool));
        if ledger.try_add(bytes) {
            return Ok(self.charge(LedgerKind::Pool(pool), bytes));
        }
        // Concurrent evictors can take bytes another sweep freed, or hold a
        // victim's gate for an instant. Sweep again while a sweep still makes
        // progress or saw busy victims; refuse once one frees nothing at all.
        for _ in 0..EVICTION_ROUNDS {
            let sweep = self.evict_for(pool, bytes, requester);
            if ledger.try_add(bytes) {
                return Ok(self.charge(LedgerKind::Pool(pool), bytes));
            }
            if sweep.evicted == 0 && sweep.busy == 0 {
                break;
            }
            if sweep.evicted == 0 {
                std::thread::yield_now();
            }
        }
        Err(WorkingSetExhausted {
            requested: bytes,
            available: ledger.available(),
        })
    }

    /// In-flight fill bytes are never evictable; saturation refuses at once.
    pub fn reserve_fill(
        self: &Arc<Self>,
        bytes: u64,
    ) -> Result<WorkingSetCharge, WorkingSetExhausted> {
        if self.in_flight.try_add(bytes) {
            Ok(self.charge(LedgerKind::InFlight, bytes))
        } else {
            Err(WorkingSetExhausted {
                requested: bytes,
                available: self.in_flight.available(),
            })
        }
    }

    fn charge(self: &Arc<Self>, ledger: LedgerKind, bytes: u64) -> WorkingSetCharge {
        WorkingSetCharge {
            accountant: self.clone(),
            ledger,
            bytes,
        }
    }

    pub fn pressure(&self) -> ResidencyPressureSnapshot {
        let mut snapshot = ResidencyPressureSnapshot {
            head_resident_bytes: self.charged_bytes(ResidencyPoolKind::Head),
            historical_resident_bytes: self.charged_bytes(ResidencyPoolKind::Historical),
            in_flight_fill_bytes: self.in_flight.charged.load(Ordering::Acquire),
            evictions_completed: self.evictions.load(Ordering::Acquire),
            busy_victim_skips: self.busy_skips.load(Ordering::Acquire),
            ..Default::default()
        };
        for cell in self.live_cells() {
            let observed = cell.working_set_observation();
            let borrowed = match observed.pool {
                Some(ResidencyPoolKind::Head) => &mut snapshot.head_borrowed_bytes,
                Some(ResidencyPoolKind::Historical) => &mut snapshot.historical_borrowed_bytes,
                None => continue,
            };
            if observed.leases > 0 {
                *borrowed = borrowed.saturating_add(observed.state_bytes);
            }
            snapshot.raw_load_cache_bytes = snapshot
                .raw_load_cache_bytes
                .saturating_add(observed.raw_cache_bytes);
        }
        snapshot
    }
}

impl WorkingSetCharge {
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn pool(&self) -> Option<ResidencyPoolKind> {
        match self.ledger {
            LedgerKind::Pool(pool) => Some(pool),
            LedgerKind::InFlight => None,
        }
    }

    pub fn is_from(&self, accountant: &Arc<WorkingSetAccountant>) -> bool {
        Arc::ptr_eq(&self.accountant, accountant)
    }

    /// Merge another charge on the same ledger into this one.
    pub fn absorb(&mut self, mut other: Self) {
        debug_assert!(Arc::ptr_eq(&self.accountant, &other.accountant));
        debug_assert_eq!(self.ledger, other.ledger);
        self.bytes += std::mem::take(&mut other.bytes);
    }

    /// Move `bytes` of this prepaid charge into a new charge without touching
    /// the ledger. Returns `None` when this charge holds fewer bytes.
    pub fn split_off(&mut self, bytes: u64) -> Option<Self> {
        self.bytes = self.bytes.checked_sub(bytes)?;
        Some(Self {
            accountant: self.accountant.clone(),
            ledger: self.ledger,
            bytes,
        })
    }
}

impl Drop for WorkingSetCharge {
    fn drop(&mut self) {
        if self.bytes != 0 {
            self.accountant.ledger(self.ledger).release(self.bytes);
        }
    }
}
