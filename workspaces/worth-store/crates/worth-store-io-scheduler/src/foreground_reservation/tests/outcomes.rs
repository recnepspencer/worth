use crate::IoSchedulerBackendCapabilityRequirement;

use super::super::*;
use super::backend_capability::backend_admission;
use super::capacity_policy::capacity_admission;
use super::foreground_case::admit_point_read_reservation;
use super::resource_budget::{full_capacity_budget, read_budget};
use super::security_scope::io_qos_security_scope_admission;

#[test]
fn certification_only_envelope_is_held_not_execution_ready() {
    let security = io_qos_security_scope_admission();
    let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
    let lane = ForegroundLaneDeclaration::point_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::certification_only_target(
            "certification-only",
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

    let outcome = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
        lane,
        &backend,
        &security,
        arbitration,
        &capacity,
    ));

    assert_eq!(outcome.state(), ForegroundReservationState::ReservationHeld);
    assert!(matches!(
        outcome.into_result(),
        Err(ForegroundReservationAdmissionDenial::CertificationOnlyEnvelopeCannotExecute)
    ));
}

#[test]
fn hard_and_soft_service_time_envelopes_are_not_executable() {
    for envelope in [
        ForegroundLatencyEnvelope::hard_bound("p99", 1),
        ForegroundLatencyEnvelope::soft_slo("p999", 1),
    ] {
        let kind = envelope.kind();
        let security = io_qos_security_scope_admission();
        let backend = backend_admission(IoSchedulerBackendCapabilityRequirement::DirectIo);
        let lane = ForegroundLaneDeclaration::point_read()
            .with_latency_envelope(envelope)
            .with_budget(read_budget());
        let arbitration =
            ForegroundArbitrationDeclaration::for_lane(ForegroundIoLaneKind::PointRead);
        let capacity = capacity_admission(
            lane,
            &backend,
            &security,
            arbitration,
            lane.requested_budget(),
            full_capacity_budget(),
        );
        let outcome = admit_foreground_reservation(ForegroundReservationAdmissionRequest::new(
            lane,
            &backend,
            &security,
            arbitration,
            &capacity,
        ));
        assert_eq!(outcome.state(), ForegroundReservationState::ReservationHeld);
        assert!(matches!(
            outcome.into_result(),
            Err(ForegroundReservationAdmissionDenial::UnsupportedServiceTimeEnvelope { kind: denied })
                if denied == kind
        ));
    }
}

#[test]
fn envelope_violation_reports_typed_cause() {
    let receipt = admit_point_read_reservation();
    let violation = receipt
        .observe_interference(3)
        .expect_err("interference above envelope must violate with cause");

    assert_eq!(violation.lane(), ForegroundIoLaneKind::PointRead);
    assert_eq!(
        violation.cause(),
        ForegroundReservationViolationCause::EnvelopeExceeded {
            allowed_interference_events: 2,
            observed_interference_events: 3,
        }
    );
}

#[test]
fn raw_shortcuts_are_typed_denials_not_reservation_authority() {
    assert_eq!(
        reject_raw_lane_label_as_foreground_reservation(),
        Err(ForegroundReservationAdmissionDenial::RawLaneLabelCannotReserve)
    );
    assert_eq!(
        reject_semantic_priority_as_foreground_reservation(),
        Err(ForegroundReservationAdmissionDenial::SemanticPriorityCannotReserve)
    );
    assert_eq!(
        reject_copied_security_scope_fields_as_foreground_reservation(),
        Err(ForegroundReservationAdmissionDenial::CopiedSecurityScopeFieldsCannotReserve)
    );
    assert_eq!(
        reject_terminal_projection_as_foreground_reservation(),
        Err(ForegroundReservationAdmissionDenial::TerminalProjectionCannotReserve)
    );
}
