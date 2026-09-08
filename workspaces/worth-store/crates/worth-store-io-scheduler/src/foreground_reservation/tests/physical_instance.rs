use super::super::{
    admit_physical_instance_foreground_reservation, ForegroundIoLaneKind,
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, ForegroundReservationAdmissionDenial,
    PhysicalInstanceForegroundAdmissionDenial, PhysicalInstanceForegroundAdmissionRequest,
};
use super::backend_capability::backend_admission;
use super::resource_budget::{full_capacity_budget, read_budget};
use super::security_scope::io_qos_security_scope_admission;
use crate::IoSchedulerBackendCapabilityRequirement;

#[test]
fn physical_instance_basis_admits_without_claiming_isolation_counters() {
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let security = io_qos_security_scope_admission();
    let lane = read_lane();
    let receipt = admit_physical_instance_foreground_reservation(
        PhysicalInstanceForegroundAdmissionRequest::new(
            lane,
            &backend,
            &security,
            full_capacity_budget(),
        ),
    )
    .expect("qualified physical instance should admit its bounded read lane");

    assert_eq!(receipt.lane(), ForegroundIoLaneKind::PointRead);
    assert_eq!(receipt.counters().requested(), read_budget());
    assert_eq!(receipt.counters().admitted_budget(), read_budget());
}

#[test]
fn physical_instance_basis_rejects_capacity_and_backend_laundering() {
    let direct = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let buffered = backend_admission(IoSchedulerBackendCapabilityRequirement::BufferedFile);
    let security = io_qos_security_scope_admission();
    let lane = read_lane();

    let capacity = admit_physical_instance_foreground_reservation(
        PhysicalInstanceForegroundAdmissionRequest::new(
            lane,
            &direct,
            &security,
            super::super::ForegroundResourceBudget::new(),
        ),
    );
    assert!(matches!(
        capacity,
        Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::InsufficientCapacity(_)
        ))
    ));

    let backend = admit_physical_instance_foreground_reservation(
        PhysicalInstanceForegroundAdmissionRequest::new(
            lane,
            &buffered,
            &security,
            full_capacity_budget(),
        ),
    );
    assert!(matches!(
        backend,
        Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::LaneBackendRequirementMismatch { .. }
        ))
    ));
}

#[test]
fn physical_instance_basis_rejects_certification_only_latency_targets() {
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let security = io_qos_security_scope_admission();
    let lane = ForegroundLaneDeclaration::point_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::certification_only_target(
            "not-executable",
        ))
        .with_budget(read_budget());
    let outcome = admit_physical_instance_foreground_reservation(
        PhysicalInstanceForegroundAdmissionRequest::new(
            lane,
            &backend,
            &security,
            full_capacity_budget(),
        ),
    );
    assert_eq!(
        outcome,
        Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::CertificationOnlyEnvelopeCannotExecute,
        ))
    );
}

#[test]
fn artifact_metadata_lane_admits_only_its_exact_queue_and_worker_shape() {
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::BufferedFile);
    let security = io_qos_security_scope_admission();
    let budget = metadata_budget();
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "artifact-metadata",
            8,
        ))
        .with_budget(budget);
    let receipt = admit_physical_instance_foreground_reservation(
        PhysicalInstanceForegroundAdmissionRequest::new(lane, &backend, &security, budget),
    )
    .expect("artifact metadata I/O should not claim range-read resources");

    assert_eq!(receipt.lane(), ForegroundIoLaneKind::ArtifactMetadataRead);
    assert_eq!(receipt.counters().admitted_budget(), budget);

    let mislabeled = ForegroundLaneDeclaration::buffered_file_internal_foreground_read()
        .unwrap()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "mislabeled-artifact-metadata",
            8,
        ))
        .with_budget(budget);
    assert!(matches!(
        admit_physical_instance_foreground_reservation(
            PhysicalInstanceForegroundAdmissionRequest::new(
                mislabeled, &backend, &security, budget,
            ),
        ),
        Err(PhysicalInstanceForegroundAdmissionDenial::Foreground(
            ForegroundReservationAdmissionDenial::MissingRequiredResourceUnit {
                lane: ForegroundIoLaneKind::InternalForegroundRead,
                ..
            }
        ))
    ));
}

fn read_lane() -> ForegroundLaneDeclaration {
    ForegroundLaneDeclaration::point_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "physical-record-read",
            8,
        ))
        .with_budget(read_budget())
}

fn metadata_budget() -> super::super::ForegroundResourceBudget {
    use super::super::{QueueSlot, WorkerPermit};
    super::super::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).unwrap())
        .with_worker_permits(WorkerPermit::new(1).unwrap())
}

#[test]
fn competing_background_reservations_preserve_live_foreground_headroom() {
    use super::super::{
        ForegroundResourceBudget, PhysicalInstanceForegroundCapacity, QueueSlot, WorkerPermit,
    };
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::BufferedFile);
    let security = io_qos_security_scope_admission();
    let configured = ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(2).unwrap())
        .with_worker_permits(WorkerPermit::new(2).unwrap());
    let capacity = PhysicalInstanceForegroundCapacity::new(configured).unwrap();
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "live-background",
            1,
        ))
        .with_budget(metadata_budget());
    let start = std::sync::Barrier::new(2);
    let results = std::thread::scope(|threads| {
        let first = threads.spawn(|| {
            start.wait();
            capacity.reserve_background(lane, &backend, &security)
        });
        let second = threads.spawn(|| {
            start.wait();
            capacity.reserve_background(lane, &backend, &security)
        });
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(capacity.snapshot().active_reservations(), 1);
    let foreground = capacity
        .reserve(lane, &backend, &security)
        .expect("ordinary work uses protected headroom while scrub is live");
    assert_eq!(capacity.snapshot().active_reservations(), 2);
    drop(results);
    assert_eq!(capacity.snapshot().active_reservations(), 1);
    drop(foreground);
    assert_eq!(capacity.snapshot().available(), configured);
    assert_eq!(capacity.snapshot().released_reservations(), 2);
}
