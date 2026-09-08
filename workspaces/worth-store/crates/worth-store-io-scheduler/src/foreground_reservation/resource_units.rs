pub use crate::resource_units::{
    BandwidthToken, CacheResidencyHint, DirtyPageBudget, FlushPermit,
    IoResourceUnitDenial as ForegroundResourceUnitDenial,
    IoResourceUnitKind as ForegroundResourceUnitKind, QueueSlot, ReadAheadWindow, ReclaimPermit,
    SyncDebt, WorkerPermit, WriteBackWindow,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ForegroundResourceBudget {
    queue_slots: u64,
    bandwidth_tokens: u64,
    flush_permits: u64,
    sync_debt: u64,
    read_ahead_window: u64,
    write_back_window: u64,
    dirty_page_budget: u64,
    worker_permits: u64,
    cache_residency_hints: u64,
    reclaim_permits: u64,
}

impl ForegroundResourceBudget {
    pub(crate) const fn half_rounded_up(self) -> Self {
        Self {
            queue_slots: self.queue_slots.div_ceil(2),
            bandwidth_tokens: self.bandwidth_tokens.div_ceil(2),
            flush_permits: self.flush_permits.div_ceil(2),
            sync_debt: self.sync_debt.div_ceil(2),
            read_ahead_window: self.read_ahead_window.div_ceil(2),
            write_back_window: self.write_back_window.div_ceil(2),
            dirty_page_budget: self.dirty_page_budget.div_ceil(2),
            worker_permits: self.worker_permits.div_ceil(2),
            cache_residency_hints: self.cache_residency_hints.div_ceil(2),
            reclaim_permits: self.reclaim_permits.div_ceil(2),
        }
    }

    pub const fn new() -> Self {
        Self {
            queue_slots: 0,
            bandwidth_tokens: 0,
            flush_permits: 0,
            sync_debt: 0,
            read_ahead_window: 0,
            write_back_window: 0,
            dirty_page_budget: 0,
            worker_permits: 0,
            cache_residency_hints: 0,
            reclaim_permits: 0,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.queue_slots == 0
            && self.bandwidth_tokens == 0
            && self.flush_permits == 0
            && self.sync_debt == 0
            && self.read_ahead_window == 0
            && self.write_back_window == 0
            && self.dirty_page_budget == 0
            && self.worker_permits == 0
            && self.cache_residency_hints == 0
            && self.reclaim_permits == 0
    }

    pub const fn denied_unit(unit: ForegroundResourceUnitKind, amount: u64) -> Self {
        match unit {
            ForegroundResourceUnitKind::QueueSlot => Self {
                queue_slots: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::BandwidthToken => Self {
                bandwidth_tokens: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::FlushPermit => Self {
                flush_permits: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::SyncDebt => Self {
                sync_debt: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::ReadAheadWindow => Self {
                read_ahead_window: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::WriteBackWindow => Self {
                write_back_window: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::DirtyPageBudget => Self {
                dirty_page_budget: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::WorkerPermit => Self {
                worker_permits: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::CacheResidencyHint => Self {
                cache_residency_hints: amount,
                ..Self::new()
            },
            ForegroundResourceUnitKind::ReclaimPermit => Self {
                reclaim_permits: amount,
                ..Self::new()
            },
        }
    }

    pub const fn amount_for(self, unit: ForegroundResourceUnitKind) -> u64 {
        match unit {
            ForegroundResourceUnitKind::QueueSlot => self.queue_slots,
            ForegroundResourceUnitKind::BandwidthToken => self.bandwidth_tokens,
            ForegroundResourceUnitKind::FlushPermit => self.flush_permits,
            ForegroundResourceUnitKind::SyncDebt => self.sync_debt,
            ForegroundResourceUnitKind::ReadAheadWindow => self.read_ahead_window,
            ForegroundResourceUnitKind::WriteBackWindow => self.write_back_window,
            ForegroundResourceUnitKind::DirtyPageBudget => self.dirty_page_budget,
            ForegroundResourceUnitKind::WorkerPermit => self.worker_permits,
            ForegroundResourceUnitKind::CacheResidencyHint => self.cache_residency_hints,
            ForegroundResourceUnitKind::ReclaimPermit => self.reclaim_permits,
        }
    }

    pub const fn with_queue_slots(mut self, unit: QueueSlot) -> Self {
        self.queue_slots = unit.get();
        self
    }

    pub const fn with_bandwidth(mut self, unit: BandwidthToken) -> Self {
        self.bandwidth_tokens = unit.get();
        self
    }

    pub const fn with_flush_permits(mut self, unit: FlushPermit) -> Self {
        self.flush_permits = unit.get();
        self
    }

    pub const fn with_sync_debt(mut self, unit: SyncDebt) -> Self {
        self.sync_debt = unit.get();
        self
    }

    pub const fn with_read_ahead(mut self, unit: ReadAheadWindow) -> Self {
        self.read_ahead_window = unit.get();
        self
    }

    pub const fn with_write_back(mut self, unit: WriteBackWindow) -> Self {
        self.write_back_window = unit.get();
        self
    }

    pub const fn with_dirty_pages(mut self, unit: DirtyPageBudget) -> Self {
        self.dirty_page_budget = unit.get();
        self
    }

    pub const fn with_worker_permits(mut self, unit: WorkerPermit) -> Self {
        self.worker_permits = unit.get();
        self
    }

    pub const fn with_cache_residency(mut self, unit: CacheResidencyHint) -> Self {
        self.cache_residency_hints = unit.get();
        self
    }

    pub const fn with_reclaim_permits(mut self, unit: ReclaimPermit) -> Self {
        self.reclaim_permits = unit.get();
        self
    }

    pub const fn queue_slots(self) -> u64 {
        self.queue_slots
    }

    pub const fn bandwidth_tokens(self) -> u64 {
        self.bandwidth_tokens
    }

    pub const fn flush_permits(self) -> u64 {
        self.flush_permits
    }

    pub const fn sync_debt(self) -> u64 {
        self.sync_debt
    }

    pub const fn read_ahead_window(self) -> u64 {
        self.read_ahead_window
    }

    pub const fn write_back_window(self) -> u64 {
        self.write_back_window
    }

    pub const fn dirty_page_budget(self) -> u64 {
        self.dirty_page_budget
    }

    pub const fn worker_permits(self) -> u64 {
        self.worker_permits
    }

    pub const fn cache_residency_hints(self) -> u64 {
        self.cache_residency_hints
    }

    pub const fn reclaim_permits(self) -> u64 {
        self.reclaim_permits
    }

    pub(crate) fn checked_sub(self, reserved: Self) -> Option<Self> {
        Some(Self {
            queue_slots: self.queue_slots.checked_sub(reserved.queue_slots)?,
            bandwidth_tokens: self
                .bandwidth_tokens
                .checked_sub(reserved.bandwidth_tokens)?,
            flush_permits: self.flush_permits.checked_sub(reserved.flush_permits)?,
            sync_debt: self.sync_debt.checked_sub(reserved.sync_debt)?,
            read_ahead_window: self
                .read_ahead_window
                .checked_sub(reserved.read_ahead_window)?,
            write_back_window: self
                .write_back_window
                .checked_sub(reserved.write_back_window)?,
            dirty_page_budget: self
                .dirty_page_budget
                .checked_sub(reserved.dirty_page_budget)?,
            worker_permits: self.worker_permits.checked_sub(reserved.worker_permits)?,
            cache_residency_hints: self
                .cache_residency_hints
                .checked_sub(reserved.cache_residency_hints)?,
            reclaim_permits: self.reclaim_permits.checked_sub(reserved.reclaim_permits)?,
        })
    }

    pub(crate) fn checked_add(self, released: Self) -> Option<Self> {
        Some(Self {
            queue_slots: self.queue_slots.checked_add(released.queue_slots)?,
            bandwidth_tokens: self
                .bandwidth_tokens
                .checked_add(released.bandwidth_tokens)?,
            flush_permits: self.flush_permits.checked_add(released.flush_permits)?,
            sync_debt: self.sync_debt.checked_add(released.sync_debt)?,
            read_ahead_window: self
                .read_ahead_window
                .checked_add(released.read_ahead_window)?,
            write_back_window: self
                .write_back_window
                .checked_add(released.write_back_window)?,
            dirty_page_budget: self
                .dirty_page_budget
                .checked_add(released.dirty_page_budget)?,
            worker_permits: self.worker_permits.checked_add(released.worker_permits)?,
            cache_residency_hints: self
                .cache_residency_hints
                .checked_add(released.cache_residency_hints)?,
            reclaim_permits: self.reclaim_permits.checked_add(released.reclaim_permits)?,
        })
    }
}
