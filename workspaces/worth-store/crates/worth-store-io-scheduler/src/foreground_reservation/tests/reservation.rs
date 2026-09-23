use crate::{
    admit_secure_frame_backend_capability_for_scheduler_claim,
    IoSchedulerBackendCapabilityRequirement,
};

use super::super::*;
use super::backend_capability::{admitted_backend_witness, backend_admission};
use super::capacity_policy::{capacity_admission, policy_receipt};
use super::foreground_case::{admit_point_read_reservation, point_read_lane};
use super::resource_budget::{full_capacity_budget, read_budget};
use super::security_scope::io_qos_security_scope_admission;

#[test]
fn foreground_reservation_admits_with_backend_security_and_envelope() {
    let security = io_qos_security_scope_admission();
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let lane = point_read_lane();
    let arbitration = ForegroundArbitrationDeclaration::for_lane(lane.lane());
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );

    let receipt = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ))
    .into_result()
    .expect("complete foreground basis should admit");

    assert_eq!(
        receipt.state(),
        ForegroundReservationState::ReservationAdmitted
    );
    assert_eq!(receipt.lane(), ForegroundIoLaneKind::PointRead);
    assert_eq!(
        receipt.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::DirectIo
    );
    assert_eq!(
        receipt.security_scope_identity(),
        security.permission().identity()
    );
}

#[test]
fn reservation_parity_preserves_equivalent_witness_fields() {
    let left = admit_point_read_reservation();
    let right = admit_point_read_reservation();

    assert_eq!(left.lane(), right.lane());
    assert_eq!(left.backend_requirement(), right.backend_requirement());
    assert_eq!(left.backend_profile(), right.backend_profile());
    assert_eq!(
        left.backend_evidence_class(),
        right.backend_evidence_class()
    );
    assert_eq!(left.envelope(), right.envelope());
    assert_eq!(left.counters(), right.counters());
    assert_eq!(
        left.security_scope_identity(),
        right.security_scope_identity()
    );
}

#[test]
fn foreground_capacity_pressure_denies_before_reservation_receipt() {
    let denial =
        admit_foreground_reservation_capacity(ForegroundReservationCapacityAdmissionRequest::new(
            point_read_lane(),
            ForegroundReservationCapacityBasis::new(
                &backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo),
                &io_qos_security_scope_admission(),
            ),
            ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::PointRead),
            ForegroundResourceBudget::new(),
            ForegroundResourceBudget::new(),
            policy_receipt(
                point_read_lane().requested_budget(),
                ForegroundResourceBudget::new(),
            ),
        ))
        .expect_err("missing requested queue and bandwidth capacity must deny");

    assert_eq!(
        denial,
        ForegroundReservationCapacityAdmissionDenial::InsufficientCapacity(
            ForegroundReservationResourceShortfall::QueueSlot {
                requested: 1,
                available: 0,
            },
        )
    );
}

#[test]
fn reservation_request_consumes_sealed_capacity_admission() {
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let security = io_qos_security_scope_admission();
    let lane = point_read_lane();
    let arbitration = ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::PointRead);
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );
    let receipt = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ))
    .into_result()
    .expect("sealed capacity admission should allow reservation");

    assert_eq!(
        receipt.counters().requested(),
        point_read_lane().requested_budget()
    );
    assert_eq!(
        receipt.counters().admitted_budget(),
        capacity.admitted_budget()
    );
    assert_eq!(
        receipt.counters().available(),
        capacity.assumed_backend_limits()
    );
}

#[test]
fn arbitrary_lane_backend_remap_cannot_mint_store_lane_contract() {
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::Fsync);
    let security = io_qos_security_scope_admission();
    let lane = ForegroundLaneDeclaration::point_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "point-read",
            2,
        ))
        .with_budget(read_budget());
    let arbitration = ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::PointRead);
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );
    let denial = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ))
    .into_result()
    .expect_err("point read cannot launder WAL fsync backend authority");

    assert_eq!(
        denial,
        ForegroundReservationAdmissionDenial::LaneBackendRequirementMismatch {
            lane_required: IoSchedulerBackendCapabilityRequirement::DirectIo,
            admitted: IoSchedulerBackendCapabilityRequirement::Fsync,
        }
    );
}

#[test]
fn secure_frame_reservation_requires_bound_security_scope() {
    let security = io_qos_security_scope_admission();
    let witness = admitted_backend_witness();
    let backend =
        admit_secure_frame_backend_capability_for_scheduler_claim(&witness, &security).unwrap();
    let lane = ForegroundLaneDeclaration::secure_frame_internal_foreground_read()
        .unwrap()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "secure-frame-foreground",
            2,
        ))
        .with_budget(full_capacity_budget());
    let arbitration =
        ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::InternalForegroundRead);
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );

    let receipt = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ))
    .into_result()
    .expect("secure-frame reservation should admit through S.5.1 scope handoff");

    assert_eq!(
        receipt.security_scope_identity(),
        security.permission().identity()
    );
    assert_eq!(
        receipt.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::SecureFrameIo
    );
}

#[test]
fn lane_specific_missing_resource_denies_before_capacity_accounting() {
    let security = io_qos_security_scope_admission();
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::Fsync);
    let lane = ForegroundLaneDeclaration::commit_critical_wal_write()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "wal-commit",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_bandwidth(BandwidthToken::bytes(4096).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    let arbitration =
        ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::CommitCriticalWalWrite);
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );

    let denial = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ))
    .into_result()
    .expect_err("WAL commit must name flush permit and sync debt");

    assert_eq!(
        denial,
        ForegroundReservationAdmissionDenial::MissingRequiredResourceUnit {
            lane: ForegroundIoLaneKind::CommitCriticalWalWrite,
            unit: ForegroundResourceUnitKind::FlushPermit,
        }
    );
}

#[test]
fn every_foreground_lane_has_an_explicit_fairness_class_without_identity_laundering() {
    let lanes = [
        ForegroundIoLaneKind::PointRead,
        ForegroundIoLaneKind::RangeRead,
        ForegroundIoLaneKind::CommitCriticalWalAppend,
        ForegroundIoLaneKind::CommitCriticalWalWrite,
        ForegroundIoLaneKind::RootPublication,
        ForegroundIoLaneKind::OrdinaryPageWrite,
        ForegroundIoLaneKind::InteractiveRead,
        ForegroundIoLaneKind::InternalForegroundRead,
        ForegroundIoLaneKind::ArtifactMetadataRead,
    ];

    let classes = lanes.map(ForegroundArbitrationPolicy::class_for);
    assert_eq!(
        classes,
        [
            ForegroundFairnessClass::PointRead,
            ForegroundFairnessClass::RangeRead,
            ForegroundFairnessClass::CommitCriticalWalWrite,
            ForegroundFairnessClass::CommitCriticalWalWrite,
            ForegroundFairnessClass::RootPublication,
            ForegroundFairnessClass::OrdinaryPageWrite,
            ForegroundFairnessClass::InteractiveRead,
            ForegroundFairnessClass::InternalForegroundRead,
            ForegroundFairnessClass::ArtifactMetadataRead,
        ],
    );

    assert_eq!(
        ForegroundIoLaneKind::CommitCriticalWalAppend.default_backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::BufferedFile,
    );
    assert_eq!(
        ForegroundIoLaneKind::CommitCriticalWalWrite.default_backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::Fsync,
    );
    assert_ne!(
        ForegroundIoLaneKind::CommitCriticalWalAppend,
        ForegroundIoLaneKind::CommitCriticalWalWrite,
    );
    assert_eq!(
        ForegroundArbitrationPolicy::reject_priority_laundering(
            ForegroundIoLaneKind::CommitCriticalWalAppend,
            ForegroundIoLaneKind::CommitCriticalWalWrite,
        ),
        Err(ForegroundFairnessDenial::PriorityLaundering {
            declared: ForegroundIoLaneKind::CommitCriticalWalAppend,
            attempted: ForegroundIoLaneKind::CommitCriticalWalWrite,
        })
    );

    assert_eq!(
        ForegroundArbitrationPolicy::reject_priority_laundering(
            ForegroundIoLaneKind::PointRead,
            ForegroundIoLaneKind::InteractiveRead,
        ),
        Err(ForegroundFairnessDenial::PriorityLaundering {
            declared: ForegroundIoLaneKind::PointRead,
            attempted: ForegroundIoLaneKind::InteractiveRead,
        })
    );
}

#[test]
fn root_publication_actions_have_non_interchangeable_filesystem_requirements() {
    let candidate = ForegroundLaneDeclaration::root_candidate_synchronization().unwrap();
    let replacement = ForegroundLaneDeclaration::root_catalog_replacement().unwrap();
    let namespace = ForegroundLaneDeclaration::root_namespace_synchronization().unwrap();

    assert_eq!(candidate.lane(), ForegroundIoLaneKind::RootPublication);
    assert_eq!(replacement.lane(), ForegroundIoLaneKind::RootPublication);
    assert_eq!(namespace.lane(), ForegroundIoLaneKind::RootPublication);
    assert_eq!(
        candidate.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedFsync
    );
    assert_eq!(
        replacement.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDurableRename
    );
    assert_eq!(
        namespace.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDirectorySync
    );
}

#[test]
fn admission_consumes_arbitration_and_denies_priority_laundering() {
    let security = io_qos_security_scope_admission();
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let lane = point_read_lane();
    let arbitration = ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::PointRead);
    let capacity = capacity_admission(
        lane,
        &backend,
        &security,
        arbitration,
        lane.requested_budget(),
        full_capacity_budget(),
    );

    let denial = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::InteractiveRead),
        &capacity,
    ))
    .into_result()
    .expect_err("reservation admission must reject foreground priority laundering");

    assert_eq!(
        denial,
        ForegroundReservationAdmissionDenial::ForegroundPriorityLaundering {
            declared: ForegroundIoLaneKind::PointRead,
            attempted: ForegroundIoLaneKind::InteractiveRead,
        }
    );
}
