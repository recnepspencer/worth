pub use worth_store::physical_runtime::RecordSchedulerReservationDenial;
pub use worth_store_physical_backend::MediaOperationRole;
pub use {
    worth_store_io_scheduler::foreground_reservation::BandwidthToken,
    worth_store_io_scheduler::foreground_reservation::DirtyPageBudget,
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
    worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope,
    worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelopeKind,
    worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial,
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
    worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    worth_store_io_scheduler::foreground_reservation::QueueSlot,
    worth_store_io_scheduler::foreground_reservation::WorkerPermit,
    worth_store_io_scheduler::foreground_reservation::WriteBackWindow,
};

pub use crate::physical_record_journeys::maintenance_interference::initialize;

pub fn page_write(envelope: ForegroundLatencyEnvelope) -> ForegroundLaneDeclaration {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(envelope)
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(worth_store_io_scheduler::foreground_reservation::WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(worth_store_io_scheduler::foreground_reservation::DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        )
}
