pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::foreground_saturation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile};
pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::RecordSchedulerReservationDenial};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};

pub(crate) fn page_write_lane() -> ForegroundLaneDeclaration {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "retained-background-head",
            2,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(worth_store_io_scheduler::foreground_reservation::WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(worth_store_io_scheduler::foreground_reservation::DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        )
}