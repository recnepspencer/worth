pub(super) fn foreground_capacity(
    capacity: crate::physical_runtime::PhysicalWorkCapacity,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, CacheResidencyHint, DirtyPageBudget, FlushPermit, QueueSlot,
        ReadAheadWindow, ReclaimPermit, SyncDebt, WorkerPermit, WriteBackWindow,
    };
    let commands = u64::try_from(capacity.commands()).expect("usize fits the scheduler counter");
    let bytes =
        u64::try_from(capacity.total_semantic_bytes()).expect("usize fits the scheduler counter");
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(commands).expect("work capacity is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("semantic capacity is nonzero"))
        .with_flush_permits(FlushPermit::new(commands).expect("work capacity is nonzero"))
        .with_sync_debt(SyncDebt::units(commands).expect("work capacity is nonzero"))
        .with_read_ahead(ReadAheadWindow::pages(commands).expect("work capacity is nonzero"))
        .with_write_back(WriteBackWindow::pages(commands).expect("work capacity is nonzero"))
        .with_dirty_pages(DirtyPageBudget::pages(commands).expect("work capacity is nonzero"))
        .with_worker_permits(WorkerPermit::new(commands).expect("work capacity is nonzero"))
        .with_cache_residency(
            CacheResidencyHint::frames(commands).expect("work capacity is nonzero"),
        )
        .with_reclaim_permits(ReclaimPermit::new(commands).expect("work capacity is nonzero"))
}