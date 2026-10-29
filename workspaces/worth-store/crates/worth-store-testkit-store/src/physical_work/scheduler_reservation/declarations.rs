pub use {worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::CacheResidencyHint, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::ReadAheadWindow, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};

pub fn reserved_buffered_file_read(
    serving: &ServingPhysicalRuntime,
) -> PhysicalInstanceForegroundReservation {
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::buffered_file_internal_foreground_read()
        .expect("buffered file is a Store-owned foreground lane")
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "physical-work-courtroom-read",
            2,
        ))
        .with_budget(read_budget());
    serving
        .reserve_physical_scheduler_foreground(lane)
        .expect("installed scheduler capacity should reserve the courtroom read")
        .0
}

pub fn reserved_page_write(
    serving: &ServingPhysicalRuntime,
) -> PhysicalInstanceForegroundReservation {
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "physical-work-courtroom-write",
            2,
        ))
        .with_budget(write_budget());
    serving
        .reserve_physical_scheduler_foreground(lane)
        .expect("installed scheduler capacity should reserve the courtroom write")
        .0
}

pub fn read_budget() -> ForegroundResourceBudget {
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
        .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
        .with_read_ahead(worth_store_io_scheduler::foreground_reservation::ReadAheadWindow::pages(1).unwrap())
        .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap())
        .with_cache_residency(worth_store_io_scheduler::foreground_reservation::CacheResidencyHint::frames(1).unwrap())
}

pub fn write_budget() -> ForegroundResourceBudget {
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
        .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
        .with_write_back(worth_store_io_scheduler::foreground_reservation::WriteBackWindow::pages(1).unwrap())
        .with_dirty_pages(worth_store_io_scheduler::foreground_reservation::DirtyPageBudget::pages(1).unwrap())
        .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap())
}