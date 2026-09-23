use worth_store::physical_runtime::RecordSchedulerReservationDenial;
use worth_store_io_scheduler::foreground_reservation::{
    BandwidthToken, DirtyPageBudget, ForegroundLaneDeclaration, ForegroundLatencyEnvelope,
    ForegroundLatencyEnvelopeKind, ForegroundReservationAdmissionDenial, ForegroundResourceBudget,
    PhysicalInstanceForegroundAdmissionDenial, QueueSlot, WorkerPermit, WriteBackWindow,
};
use worth_store_physical_backend::MediaOperationRole;

use super::initialize;

#[test]
fn hard_service_time_is_refused_before_media_and_bounded_interference_is_admitted() {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize(&parent.path().join("store"));
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write(
            ForegroundLatencyEnvelope::hard_bound("p99", 1),
        )),
        Err(RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope {
                    kind: ForegroundLatencyEnvelopeKind::HardBound,
                },
            ),
        ))
    ));
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write(
            ForegroundLatencyEnvelope::soft_slo("p999", 1),
        )),
        Err(RecordSchedulerReservationDenial::Admission(
            PhysicalInstanceForegroundAdmissionDenial::Foreground(
                ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope {
                    kind: ForegroundLatencyEnvelopeKind::SoftSlo,
                },
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving
        .reserve_physical_scheduler_foreground(page_write(
            ForegroundLatencyEnvelope::bounded_interference("page-write", 2),
        ))
        .expect("bounded interference is the executable foreground claim");
    serving.close();
}

fn page_write(envelope: ForegroundLatencyEnvelope) -> ForegroundLaneDeclaration {
    ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(envelope)
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_bandwidth(BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        )
}
