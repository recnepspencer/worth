//! Foreground resource budgets each scheduler admission reserves.

pub(super) fn read_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, CacheResidencyHint, QueueSlot, ReadAheadWindow, WorkerPermit,
    };
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("record coordinates are nonempty"))
        .with_read_ahead(ReadAheadWindow::pages(1).expect("one read-ahead page is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"))
        .with_cache_residency(CacheResidencyHint::frames(1).expect("one frame hint is nonzero"))
}

pub(super) fn metadata_budget(
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{QueueSlot, WorkerPermit};
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"))
}

pub(super) fn write_budget(
    bytes: u64,
    synchronization: bool,
    publication: bool,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, DirtyPageBudget, FlushPermit, QueueSlot, SyncDebt, WorkerPermit,
        WriteBackWindow,
    };
    let budget = worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("record coordinates are nonempty"))
        .with_write_back(WriteBackWindow::pages(1).expect("one writeback page is nonzero"))
        .with_dirty_pages(DirtyPageBudget::pages(1).expect("one dirty page is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"));
    let budget = if synchronization {
        budget.with_flush_permits(FlushPermit::new(1).expect("one flush permit is nonzero"))
    } else {
        budget
    };
    if publication {
        budget.with_sync_debt(SyncDebt::units(1).expect("one sync-debt unit is nonzero"))
    } else {
        budget
    }
}

pub(super) fn wal_append_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{BandwidthToken, QueueSlot, WorkerPermit};
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one WAL append is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("an admitted WAL frame is nonempty"))
        .with_worker_permits(WorkerPermit::new(1).expect("one WAL append is nonzero"))
}

pub(super) fn wal_barrier_budget(
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, FlushPermit, QueueSlot, SyncDebt, WorkerPermit,
    };
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one WAL barrier is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(1).expect("one barrier accounting unit is nonzero"))
        .with_flush_permits(FlushPermit::new(1).expect("one WAL barrier is nonzero"))
        .with_sync_debt(SyncDebt::units(1).expect("one WAL barrier is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one WAL barrier is nonzero"))
}
