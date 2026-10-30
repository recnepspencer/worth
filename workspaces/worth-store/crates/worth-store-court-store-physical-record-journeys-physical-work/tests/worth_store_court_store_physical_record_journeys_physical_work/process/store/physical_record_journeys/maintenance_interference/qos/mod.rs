#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::qos::*;
use worth_store::physical_runtime::RecordSchedulerReservationDenial;
use worth_store_physical_backend::MediaOperationRole;
use {
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

use worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize;

#[test]
fn hard_service_time_is_refused_before_media_and_bounded_interference_is_admitted() {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize(&parent.path().join("store"));
    let writes = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write(
            worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::hard_bound("p99", 1),
        )),
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope {
                    kind: worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelopeKind::HardBound,
                },
            ),
        ))
    ));
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write(
            worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::soft_slo("p999", 1),
        )),
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope {
                    kind: worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelopeKind::SoftSlo,
                },
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes
    );
    serving
        .reserve_physical_scheduler_foreground(page_write(
            worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference("page-write", 2),
        ))
        .expect("bounded interference is the executable foreground claim");
    serving.close();
}
